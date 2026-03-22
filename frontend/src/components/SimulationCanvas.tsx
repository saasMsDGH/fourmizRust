import { useCallback, useMemo } from 'react';
// @ts-ignore
import { Stage, Graphics, PixiComponent, useApp } from '@pixi/react';
import * as PIXI from 'pixi.js';
import { Viewport as PixiViewport } from 'pixi-viewport';

export interface ViewportProps {
  app: PIXI.Application;
  screenWidth: number;
  screenHeight: number;
  worldWidth: number;
  worldHeight: number;
  children?: React.ReactNode;
  onCameraChange?: (state: import('../types').CameraState) => void;
}

const PixiViewportComponent = PixiComponent('Viewport', {
  create: (props: ViewportProps) => {
    const viewport = new PixiViewport({
      screenWidth: props.screenWidth,
      screenHeight: props.screenHeight,
      worldWidth: props.worldWidth,
      worldHeight: props.worldHeight,
      events: props.app.renderer.events,
    });
    viewport.drag().pinch().wheel().clampZoom({ minScale: 0.02, maxScale: 3.0 });
    
    // Zoom out slightly to see the map context by default
    viewport.setZoom(0.3);
    viewport.moveCenter(props.worldWidth / 2, props.worldHeight / 2);
    
    if (props.onCameraChange) {
      const updateCamera = () => {
        // @ts-ignore
        props.onCameraChange({
          x: viewport.left,
          y: viewport.top,
          width: props.screenWidth / viewport.scale.x,
          height: props.screenHeight / viewport.scale.y,
          worldWidth: props.worldWidth,
          worldHeight: props.worldHeight,
        });
      };
      viewport.on('moved', updateCamera);
      viewport.on('zoomed', updateCamera);
      setTimeout(updateCamera, 0);
    }
    
    return viewport;
  },
});

const ViewportWrapper = (props: any) => {
  const app = useApp();
  return <PixiViewportComponent app={app} {...props} />;
};
import { AntRole, AntState, ResourceType, RoomType } from '../types';
import type { GameState, Ant, Resource, CameraState } from '../types';

interface Props {
  width: number;
  height: number;
  gameState: GameState | null;
  onCameraChange?: (state: CameraState) => void;
}

export const SimulationCanvas = ({ width, height, gameState, onCameraChange }: Props) => {
  
  // Scale factor: logic mapping
  const scaleRatio = 5; // 1 logic unit = 5 pixels (zoomed out a bit for the massive map)
  const simWidth = 3162 * scaleRatio; 
  const simHeight = 3162 * scaleRatio;

  const AntGraphic = ({ ant }: { ant: Ant }) => {
    const draw = useCallback(
      (g: PIXI.Graphics) => {
        g.clear();
        
        if (ant.role === AntRole.QUEEN) {
          g.beginFill(0x9333ea, 1);
          const size = 12;
          const hexPath = [
            0, -size,
            size * 0.866, -size / 2,
            size * 0.866, size / 2,
            0, size,
            -size * 0.866, size / 2,
            -size * 0.866, -size / 2,
          ];
          g.drawPolygon(hexPath);
          g.endFill();
        } else if (ant.role === AntRole.SOLDIER) {
          // Soldier Anatomy: Big head, Mandibles, Sturdy thorax
          g.beginFill(0x78350f, 1); 
          g.drawEllipse(12, 0, 8, 6); // Huge Head
          
          g.lineStyle(2, 0x000000); // Mandibles
          g.moveTo(18, -3); g.lineTo(24, -6); g.lineTo(22, -2);
          g.moveTo(18, 3); g.lineTo(24, 6); g.lineTo(22, 2);
          g.lineStyle(0);

          g.beginFill(0x451a03, 1); 
          g.drawEllipse(2, 0, 7, 5); // Thorax
          g.beginFill(0x271001, 1);
          g.drawEllipse(-8, 0, 8, 6); // Abdomen
          g.endFill();

          g.lineStyle(2, 0x271001); // Legs
          g.moveTo(2, -5); g.lineTo(0, -12);
          g.moveTo(2, 5); g.lineTo(0, 12);
          g.moveTo(-2, -5); g.lineTo(-4, -12);
          g.moveTo(-2, 5); g.lineTo(-4, 12);
          g.lineStyle(0);
        } else {
          // Worker Anatomy
          g.beginFill(0x451a03, 1);
          g.drawCircle(8, 0, 3);  // Head
          g.drawCircle(0, 0, 4);  // Thorax
          g.drawCircle(-8, 0, 5); // Abdomen
          g.endFill();

          g.lineStyle(1, 0x271001); // Legs
          g.moveTo(0, -4); g.lineTo(2, -8);
          g.moveTo(0, 4); g.lineTo(2, 8);
          g.moveTo(-2, -4); g.lineTo(-4, -8);
          g.moveTo(-2, 4); g.lineTo(-4, 8);
          g.lineStyle(0);
        }

        if (ant.state === AntState.RETURNING_WITH_FOOD) {
          g.beginFill(0x22c55e, 1);
          g.drawCircle(14, 0, 3);
          g.endFill();
        }
      },
      [ant.role, ant.state]
    );

    return <Graphics 
             draw={draw} 
             x={ant.x * scaleRatio} 
             y={ant.y * scaleRatio} 
             rotation={ant.angle || 0} 
           />;
  };

  const ResourceGraphic = ({ resource }: { resource: Resource }) => {
    const draw = useCallback(
      (g: PIXI.Graphics) => {
        g.clear();
        
        const ratio = Math.max(0.1, resource.quantity / 30); // max 30 per cell
        const alpha = 0.5 + (ratio * 0.5);
        const size = 15 * ratio;

        const color = resource.type === ResourceType.PLANT ? 0x10b981 : 0xef4444; 

        g.beginFill(color, alpha);
        g.drawCircle(0, 0, size);
        g.drawCircle(size*0.4, size*0.4, size*0.8);
        g.drawCircle(-size*0.3, size*0.5, size*0.6);
        g.endFill();
      },
      [resource.quantity, resource.type]
    );

    return <Graphics draw={draw} x={resource.x * scaleRatio} y={resource.y * scaleRatio} />;
  };

  const drawBackgroundAndNests = useCallback((g: PIXI.Graphics) => {
    g.clear();
    
    // Dirt Background
    g.beginFill(0xD2B48C, 1); 
    g.drawRect(0, 0, simWidth, simHeight);
    g.endFill();

    // Draw Generative Nests
    if (gameState?.nests) {
      for (const nest of gameState.nests) {
        
        // Find royal chamber as center for tunnels
        const royal = nest.rooms.find(r => r.room_type === RoomType.ROYAL);
        
        if (royal) {
          // Draw connecting tunnels
          g.lineStyle(20, 0x2a1c14); // Tunnel width
          for (const room of nest.rooms) {
            if (room.id !== royal.id) {
              g.moveTo(royal.x * scaleRatio, royal.y * scaleRatio);
              g.lineTo(room.x * scaleRatio, room.y * scaleRatio);
            }
          }
          g.lineStyle(0);
        }

        // Draw Rooms
        for (const room of nest.rooms) {
          let rcolor = 0x2a1c14;
          if (room.room_type === RoomType.ROYAL) rcolor = 0x3b2046; // Purple tint for Queen
          if (room.room_type === RoomType.GRANARY) rcolor = 0x293120; // Green tint for Food
          if (room.room_type === RoomType.NURSERY) rcolor = 0x463520; // Amber tint for Nursery

          g.beginFill(rcolor, 1);
          g.drawCircle(room.x * scaleRatio, room.y * scaleRatio, room.radius * scaleRatio);
          
          g.beginFill(0x1a110a, 0.4); // Inner shadow
          g.drawCircle(room.x * scaleRatio, room.y * scaleRatio, room.radius * scaleRatio * 0.8);
          g.endFill();
        }
      }
    }

  }, [gameState?.nests, simWidth, simHeight, scaleRatio]);

  const ants = useMemo(() => gameState?.ants || [], [gameState?.ants]);
  const resources = useMemo(() => gameState?.resources || [], [gameState?.resources]);

  // Custom transform state removed in favor of pixi-viewport

  return (
    <div style={{ width: '100%', height: '100%' }}>
        <Stage 
          width={width} 
          height={height} 
          options={{ backgroundColor: 0xD2B48C, antialias: true, autoDensity: true, resolution: window.devicePixelRatio || 1 }}
        >
          <ViewportWrapper 
             screenWidth={width}
             screenHeight={height}
             worldWidth={simWidth}
             worldHeight={simHeight}
             onCameraChange={onCameraChange}
          >
              <Graphics draw={drawBackgroundAndNests} />

              {/* Render Resources */}
              {resources.map((res) => (
                 <ResourceGraphic key={`res-${res.id}`} resource={res} />
              ))}

              {/* Render Ants */}
              {ants.map((ant) => (
                <AntGraphic key={`ant-${ant.id}`} ant={ant} />
              ))}
          </ViewportWrapper>
        </Stage>
    </div>
  );
};
