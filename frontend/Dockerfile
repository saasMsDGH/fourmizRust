FROM node:20-alpine as builder

WORKDIR /app
COPY package*.json ./
# Use legacy peer deps purely because of current React/PixiJS dependency mismatches
RUN npm install --legacy-peer-deps

COPY . .
RUN npm run build

FROM nginx:alpine
# Copy built static assets
COPY --from=builder /app/dist /usr/share/nginx/html

# Write basic SPA configuration for Nginx to handle pushState/React Router
RUN echo 'server {\n\
    listen 80;\n\
    location / {\n\
        root /usr/share/nginx/html;\n\
        index index.html;\n\
        try_files $uri $uri/ /index.html;\n\
    }\n\
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
