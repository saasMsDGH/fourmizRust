import { useCallback, useMemo, useRef, useState, useEffect } from 'react';
// @ts-ignore
import { Stage, Graphics, Container } from '@pixi/react';
import * as PIXI from 'pixi.js';
import { AntRole, AntState, ResourceType, RoomType } from '../types';
import type { GameState, Ant, Resource } from '../types';

interface Props {
  width: number;
  height: number;
  gameState: GameState | null;
}

export const SimulationCanvas = ({ width, height, gameState }: Props) => {
  
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
          g.drawEllipse(0, -15, 6, 8); // Head
          g.beginFill(0x7e22ce, 1);
          g.drawEllipse(0, 0, 8, 10); // Thorax
          g.beginFill(0x581c87, 1);
          g.drawEllipse(0, 20, 14, 22); // Abdomen
          g.endFill();
          g.lineStyle(2, 0x4c1d95); // Legs
          g.moveTo(-8, 0); g.lineTo(-20, -5);
          g.moveTo(8, 0); g.lineTo(20, -5);
          g.moveTo(-8, 5); g.lineTo(-22, 10);
          g.moveTo(8, 5); g.lineTo(22, 10);
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
          g.drawEllipse(10, 0, 4, 3); // Head
          g.beginFill(0x78350f, 1); 
          g.drawEllipse(4, 0, 5, 4); // Thorax
          g.beginFill(0x271001, 1);
          g.drawEllipse(-4, 0, 6, 5); // Abdomen
          g.endFill();

          g.lineStyle(1, 0x271001); // Legs
          g.moveTo(4, -4); g.lineTo(2, -8);
          g.moveTo(4, 4); g.lineTo(2, 8);
          g.moveTo(0, -4); g.lineTo(-2, -8);
          g.moveTo(0, 4); g.lineTo(-2, 8);
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
    g.beginFill(0x1a1614, 1); 
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

  const [transform, setTransform] = useState({ x: 0, y: 0, scale: 0.1 });
  const [isDragging, setIsDragging] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);
  const lastPos = useRef({ x: 0, y: 0 });

  // Center on Queen or nest on first load
  useEffect(() => {
    if (!isInitialized && gameState?.ants?.length) {
      const queen = gameState.ants.find(a => a.role === AntRole.QUEEN);
      const targetX = queen ? queen.x : (simWidth / scaleRatio / 2);
      const targetY = queen ? queen.y : (simHeight / scaleRatio / 2);

      const idealScale = 0.5; // Zoom in to see the ants clearly
      
      setTransform({
        x: width/2 - (targetX * scaleRatio * idealScale),
        y: height/2 - (targetY * scaleRatio * idealScale),
        scale: idealScale
      });
      setIsInitialized(true);
    }
  }, [gameState?.ants, isInitialized, width, height]);

  const onPointerDown = (e: any) => {
    setIsDragging(true);
    lastPos.current = { x: e.clientX, y: e.clientY };
  };

  const onPointerMove = (e: any) => {
    if (isDragging) {
      const dx = e.clientX - lastPos.current.x;
      const dy = e.clientY - lastPos.current.y;
      setTransform((prev: any) => ({ ...prev, x: prev.x + dx, y: prev.y + dy }));
      lastPos.current = { x: e.clientX, y: e.clientY };
    }
  };

  const onPointerUp = () => {
    setIsDragging(false);
  };

  const onWheel = (e: any) => {
     const zoomFactor = -e.deltaY * 0.0005;
     let newScale = Math.max(0.02, Math.min(3.0, transform.scale + zoomFactor));
     
     const mouseX = e.clientX;
     const mouseY = e.clientY;
     
     const newX = mouseX - (mouseX - transform.x) * (newScale / transform.scale);
     const newY = mouseY - (mouseY - transform.y) * (newScale / transform.scale);
     
     setTransform({ x: newX, y: newY, scale: newScale });
  };

  return (
    <div 
        onPointerDown={onPointerDown} 
        onPointerMove={onPointerMove} 
        onPointerUp={onPointerUp} 
        onPointerOut={onPointerUp}
        onWheel={onWheel}
        style={{ width: '100%', height: '100%' }}
    >
        <Stage 
          width={width} 
          height={height} 
          options={{ backgroundColor: 0x111111, antialias: true, autoDensity: true, resolution: window.devicePixelRatio || 1 }}
        >
          <Container 
             x={transform.x} 
             y={transform.y} 
             scale={transform.scale}
             interactive={true}
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
          </Container>
        </Stage>
    </div>
  );
};
