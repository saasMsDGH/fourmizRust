name: Build & Release (Tag Only)

on:
  push:
    tags:
      - "v*"

jobs:
  build-release:
    runs-on: ubuntu-latest
    permissions:
      contents: write

    steps:
      - uses: actions/checkout@v4

      - name: Extract version from tag
        id: version
        shell: bash
        run: |
          set -euo pipefail
          echo "VERSION=${GITHUB_REF_NAME#v}" >> "$GITHUB_OUTPUT"

      - name: Validate release secrets
        env:
          DOCKER_USERNAME: ${{ secrets.DOCKER_USERNAME }}
          DOCKER_PASSWORD: ${{ secrets.DOCKER_PASSWORD }}
        shell: bash
        run: |
          set -euo pipefail
          : "${DOCKER_USERNAME:?Set GitHub Actions secret DOCKER_USERNAME before tagging a release.}"
          : "${DOCKER_PASSWORD:?Set GitHub Actions secret DOCKER_PASSWORD before tagging a release.}"

      - name: Login to Docker Hub
        uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}

      - name: Build & Push Backend
        uses: docker/build-push-action@v6
        with:
          context: .
          file: ./backend/Dockerfile
          push: true
          tags: |
            ${{ secrets.DOCKER_USERNAME }}/moneyz-backend:${{ steps.version.outputs.VERSION }}
            ${{ secrets.DOCKER_USERNAME }}/moneyz-backend:latest

      - name: Build & Push Frontend
        uses: docker/build-push-action@v6
        with:
          context: ./frontend
          file: ./frontend/Dockerfile
          push: true
          build-args: |
            APP_VERSION=v${{ steps.version.outputs.VERSION }}
          tags: |
            ${{ secrets.DOCKER_USERNAME }}/moneyz-frontend:${{ steps.version.outputs.VERSION }}
            ${{ secrets.DOCKER_USERNAME }}/moneyz-frontend:latest

      - name: Build & Push Config Service
        uses: docker/build-push-action@v6
        with:
          context: .
          file: ./config-service/Dockerfile
          push: true
          tags: |
            ${{ secrets.DOCKER_USERNAME }}/moneyz-config:${{ steps.version.outputs.VERSION }}
            ${{ secrets.DOCKER_USERNAME }}/moneyz-config:latest

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          name: Moneyz v${{ steps.version.outputs.VERSION }}
          generate_release_notes: true
name: Deploy to K3s

on:
  workflow_run:
    workflows: ["Build & Release (Tag Only)"]
    types: [completed]

jobs:
  deploy:
    runs-on: [self-hosted, k8s-deploy]
    if: ${{ github.event.workflow_run.conclusion == 'success' }}

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Resolve tag (v*) from triggering SHA
        id: resolve
        shell: bash
        run: |
          set -euo pipefail
          SHA="${{ github.event.workflow_run.head_sha }}"
          git fetch --force --tags
          TAG="$(git tag --points-at "$SHA" | grep -E '^v[0-9]+' | head -n 1 || true)"
          if [[ -z "$TAG" ]]; then
            echo "No v* tag points to $SHA"
            exit 1
          fi
          echo "VERSION=${TAG#v}" >> "$GITHUB_OUTPUT"

      - name: Deploy manifests
        env:
          IMAGE_BACKEND: ${{ secrets.DOCKER_USERNAME }}/moneyz-backend:${{ steps.resolve.outputs.VERSION }}
          IMAGE_FRONTEND: ${{ secrets.DOCKER_USERNAME }}/moneyz-frontend:${{ steps.resolve.outputs.VERSION }}
          IMAGE_CONFIG: ${{ secrets.DOCKER_USERNAME }}/moneyz-config:${{ steps.resolve.outputs.VERSION }}
          APP_HOST: ${{ vars.MONEYZ_HOST }}
        shell: bash
        run: |
          set -euo pipefail
          : "${APP_HOST:?Set repository variable MONEYZ_HOST before running deploy.}"

          envsubst < k8s/moneyz.yaml > /tmp/moneyz.rendered.yaml
          envsubst < k8s/moneyz-ingressroute.yaml > /tmp/moneyz-ingressroute.rendered.yaml

          kubectl apply -f /tmp/moneyz.rendered.yaml -f /tmp/moneyz-ingressroute.rendered.yaml

          kubectl rollout status -n apps deployment/moneyz-backend
          kubectl rollout status -n apps deployment/moneyz-frontend
          kubectl rollout status -n apps deployment/moneyz-config
name: CI - Quality Check

on:
  push:
    branches: ["main"]
  pull_request:
    branches: ["main"]

jobs:
  quality-check:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Go
        uses: actions/setup-go@v5
        with:
          go-version-file: go.mod
          cache: true

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          run_install: false

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: frontend/pnpm-lock.yaml

      - name: Install frontend dependencies
        run: pnpm --dir frontend install --frozen-lockfile

      - name: Validate backend build
        run: go test ./...

      - name: Validate frontend build
        run: pnpm --dir frontend build

      - name: Docker dry run (backend)
        run: docker build -f backend/Dockerfile .

      - name: Docker dry run (frontend)
        run: docker build -f frontend/Dockerfile ./frontend



apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: moneyz-security-headers
  namespace: apps
spec:
  headers:
    frameDeny: true
    browserXssFilter: true
    contentTypeNosniff: true
    referrerPolicy: strict-origin-when-cross-origin
    permissionsPolicy: camera=(), geolocation=(), microphone=(), payment=(), usb=()
    forceSTSHeader: true
    stsSeconds: 31536000
    stsIncludeSubdomains: true
    stsPreload: true
    customResponseHeaders:
      Server: ""
---
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: moneyz-api-ratelimit
  namespace: apps
spec:
  rateLimit:
    average: 120
    burst: 60
    period: 1m
---
apiVersion: traefik.io/v1alpha1
kind: IngressRoute
metadata:
  name: moneyz
  namespace: apps
spec:
  entryPoints:
    - websecure
  routes:
    - match: Host(`${APP_HOST}`) && PathPrefix(`/api/v1/config`)
      kind: Rule
      priority: 110
      middlewares:
        - name: moneyz-security-headers
        - name: moneyz-api-ratelimit
      services:
        - name: moneyz-config
          port: 8090
    - match: Host(`${APP_HOST}`) && PathPrefix(`/api`)
      kind: Rule
      priority: 100
      middlewares:
        - name: moneyz-security-headers
        - name: moneyz-api-ratelimit
      services:
        - name: moneyz-backend
          port: 8081
    - match: Host(`${APP_HOST}`) && PathPrefix(`/swagger`)
      kind: Rule
      priority: 90
      middlewares:
        - name: moneyz-security-headers
        - name: moneyz-api-ratelimit
      services:
        - name: moneyz-backend
          port: 8081
    - match: Host(`${APP_HOST}`)
      kind: Rule
      priority: 1
      middlewares:
        - name: moneyz-security-headers
      services:
        - name: moneyz-frontend
          port: 80
  tls:
    certResolver: le_dns
    domains:
      - main: "${APP_HOST}"
apiVersion: v1
kind: Namespace
metadata:
  name: apps
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: moneyz-data
  namespace: apps
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: moneyz-backend
  namespace: apps
  labels:
    app: moneyz-backend
spec:
  replicas: 1
  selector:
    matchLabels:
      app: moneyz-backend
  template:
    metadata:
      labels:
        app: moneyz-backend
    spec:
      terminationGracePeriodSeconds: 20
      securityContext:
        runAsNonRoot: true
        runAsUser: 10001
        runAsGroup: 10001
        fsGroup: 10001
      containers:
        - name: backend
          image: ${IMAGE_BACKEND}
          imagePullPolicy: IfNotPresent
          ports:
            - containerPort: 8081
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: false
          env:
            - name: MONEYZ_ENV
              value: production
            - name: APP_HOST
              value: ${APP_HOST}
            - name: PORT
              value: "8081"
            - name: MONEYZ_DB_PATH
              value: /data/moneyz_sync.db
            - name: CORS_ALLOWED_ORIGINS
              value: https://${APP_HOST}
            - name: ENABLE_SWAGGER
              value: "false"
            - name: ENABLE_DEMO_API_ROUTES
              value: "false"
          volumeMounts:
            - name: moneyz-data
              mountPath: /data
          readinessProbe:
            httpGet:
              path: /api/v1/health
              port: 8081
            initialDelaySeconds: 3
            periodSeconds: 10
            timeoutSeconds: 2
            failureThreshold: 6
          livenessProbe:
            httpGet:
              path: /api/v1/health
              port: 8081
            initialDelaySeconds: 10
            periodSeconds: 20
            timeoutSeconds: 2
            failureThreshold: 3
          resources:
            limits:
              cpu: "500m"
              memory: "512Mi"
            requests:
              cpu: "100m"
              memory: "128Mi"
      volumes:
        - name: moneyz-data
          persistentVolumeClaim:
            claimName: moneyz-data
---
apiVersion: v1
kind: Service
metadata:
  name: moneyz-backend
  namespace: apps
spec:
  selector:
    app: moneyz-backend
  ports:
    - name: http
      port: 8081
      targetPort: 8081
      protocol: TCP
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: moneyz-config
  namespace: apps
  labels:
    app: moneyz-config
spec:
  replicas: 1
  selector:
    matchLabels:
      app: moneyz-config
  template:
    metadata:
      labels:
        app: moneyz-config
    spec:
      terminationGracePeriodSeconds: 20
      securityContext:
        runAsNonRoot: true
        runAsUser: 10001
        runAsGroup: 10001
        fsGroup: 10001
      containers:
        - name: config
          image: ${IMAGE_CONFIG}
          imagePullPolicy: IfNotPresent
          ports:
            - containerPort: 8090
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: false
          env:
            - name: PORT
              value: "8090"
          readinessProbe:
            httpGet:
              path: /health
              port: 8090
            initialDelaySeconds: 3
            periodSeconds: 10
            timeoutSeconds: 2
            failureThreshold: 6
          livenessProbe:
            httpGet:
              path: /health
              port: 8090
            initialDelaySeconds: 10
            periodSeconds: 20
            timeoutSeconds: 2
            failureThreshold: 3
          resources:
            limits:
              cpu: "250m"
              memory: "128Mi"
            requests:
              cpu: "30m"
              memory: "32Mi"
---
apiVersion: v1
kind: Service
metadata:
  name: moneyz-config
  namespace: apps
spec:
  selector:
    app: moneyz-config
  ports:
    - name: http
      port: 8090
      targetPort: 8090
      protocol: TCP
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: moneyz-frontend
  namespace: apps
  labels:
    app: moneyz-frontend
spec:
  replicas: 1
  selector:
    matchLabels:
      app: moneyz-frontend
  template:
    metadata:
      labels:
        app: moneyz-frontend
    spec:
      terminationGracePeriodSeconds: 20
      securityContext:
        runAsNonRoot: true
      containers:
        - name: frontend
          image: ${IMAGE_FRONTEND}
          imagePullPolicy: IfNotPresent
          ports:
            - containerPort: 8080
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: false
          readinessProbe:
            httpGet:
              path: /
              port: 8080
            initialDelaySeconds: 3
            periodSeconds: 10
            timeoutSeconds: 2
            failureThreshold: 6
          livenessProbe:
            httpGet:
              path: /
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 20
            timeoutSeconds: 2
            failureThreshold: 3
          resources:
            limits:
              cpu: "500m"
              memory: "256Mi"
            requests:
              cpu: "50m"
              memory: "64Mi"
---
apiVersion: v1
kind: Service
metadata:
  name: moneyz-frontend
  namespace: apps
spec:
  selector:
    app: moneyz-frontend
  ports:
    - name: http
      port: 80
      targetPort: 8080
      protocol: TCP
