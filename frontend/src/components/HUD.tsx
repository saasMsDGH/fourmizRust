import { Play, Square, RotateCcw, Activity } from 'lucide-react';
import { CommandType } from '../types';
import type { GameState, CameraState } from '../types';

interface Props {
  gameState: GameState | null;
  connected: boolean;
  onCommand: (cmd: CommandType) => void;
  cameraState?: CameraState | null;
}

export const HUD = ({ gameState, connected, onCommand, cameraState }: Props) => {
  const population = (gameState?.ants || []).filter(a => a.role === 1).length;
  const foodReserves = (gameState?.resources || []).reduce((acc, r) => acc + r.quantity, 0);
  
  // Pretend health is 100% just for UI completeness since backend doesn't model queen health yet
  const queenHealth = 100;

  return (
    <div className="absolute inset-0 pointer-events-none z-10 flex flex-col justify-between p-6 h-full">
      {/* Top Banner: Global Stats */}
      <div className="flex justify-between items-start">
        <div className="bg-slate-900/40 backdrop-blur-xl p-5 rounded-2xl border border-white/10 shadow-2xl pointer-events-auto min-w-[300px]">
          <div className="flex items-center justify-between mb-4">
            <h1 className="text-2xl font-black tracking-tight bg-gradient-to-br from-indigo-400 to-purple-400 bg-clip-text text-transparent uppercase drop-shadow-sm">
              Colony Alpha
            </h1>
            <div className="flex items-center gap-2 bg-black/30 px-3 py-1.5 rounded-full">
              <span className={`w-2.5 h-2.5 rounded-full shadow-[0_0_10px_rgba(0,0,0,0.5)] ${connected ? 'bg-emerald-500 shadow-emerald-500/50 animate-pulse' : 'bg-rose-500 shadow-rose-500/50'}`} />
              <span className="text-xs font-bold text-slate-300 tracking-wider">
                {connected ? 'LIVE' : 'OFFLINE'}
              </span>
            </div>
          </div>
          
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-black/20 p-3 rounded-xl border border-white/5">
              <p className="text-xs text-slate-400 uppercase font-semibold mb-1">Cycle</p>
              <p className="font-mono text-xl text-indigo-300 font-bold">{gameState?.tick || 0}</p>
            </div>
            <div className="bg-black/20 p-3 rounded-xl border border-white/5">
              <p className="text-xs text-slate-400 uppercase font-semibold mb-1">Population</p>
              <p className="font-mono text-xl text-amber-300 font-bold">{population}</p>
            </div>
          </div>
          
          <p className="text-[10px] text-slate-500 mt-4 uppercase tracking-widest text-center opacity-70">
            Click map to spawn resources
          </p>
        </div>

        {/* Side Panel: Focus */}
        <div className="bg-slate-900/40 backdrop-blur-xl p-5 rounded-2xl border border-white/10 shadow-2xl pointer-events-auto min-w-[250px]">
          <h2 className="text-sm text-slate-400 font-bold uppercase tracking-wider mb-4 flex items-center gap-2">
            <Activity className="w-4 h-4 text-purple-400" />
            Hive Status
          </h2>
          
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-xs mb-1.5">
                <span className="text-slate-300 font-medium">Queen Health</span>
                <span className="text-rose-400 font-mono font-bold">{queenHealth}%</span>
              </div>
              <div className="h-2 w-full bg-black/40 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-gradient-to-r from-rose-500 to-rose-400 transition-all duration-500" 
                  style={{ width: `${queenHealth}%` }}
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-xs mb-1.5">
                <span className="text-slate-300 font-medium">Global Reserves</span>
                <span className="text-emerald-400 font-mono font-bold">{foodReserves.toFixed(0)}</span>
              </div>
              <div className="h-2 w-full bg-black/40 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-gradient-to-r from-emerald-600 to-emerald-400 transition-all duration-500" 
                  style={{ width: `${Math.min(100, (foodReserves / 5000) * 100)}%` }} // Arbitrary max 5000 for visuals
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Controls */}
      <div className="flex justify-center mb-4">
        <div className="bg-slate-900/60 backdrop-blur-2xl p-2 rounded-2xl border border-white/10 shadow-[0_0_40px_rgba(0,0,0,0.3)] flex gap-2 pointer-events-auto">
          <button 
            onClick={() => onCommand(CommandType.START)}
            className="flex items-center gap-2 px-6 py-3 bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-400 rounded-xl transition-all hover:scale-105 active:scale-95 group font-bold tracking-wide border border-emerald-500/30"
          >
            <Play className="w-5 h-5 fill-current" />
            RESUME
          </button>
          
          <button 
            onClick={() => onCommand(CommandType.PAUSE)}
            className="flex items-center gap-2 px-6 py-3 bg-amber-500/20 hover:bg-amber-500/30 text-amber-400 rounded-xl transition-all hover:scale-105 active:scale-95 group font-bold tracking-wide border border-amber-500/30"
          >
            <Square className="w-5 h-5 fill-current" />
            PAUSE
          </button>
          
          <button 
            onClick={() => onCommand(CommandType.SPAWN_PLANT)}
            className="flex items-center gap-2 px-6 py-3 bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-400 rounded-xl transition-all hover:scale-105 active:scale-95 group font-bold tracking-wide border border-emerald-500/30"
          >
            SPAWN PLANT
          </button>
          
          <button 
            onClick={() => onCommand(CommandType.SPAWN_ANIMAL)}
            className="flex items-center gap-2 px-6 py-3 bg-rose-500/20 hover:bg-rose-500/30 text-rose-400 rounded-xl transition-all hover:scale-105 active:scale-95 group font-bold tracking-wide border border-rose-500/30"
          >
            SPAWN ANIMAL
          </button>
          
          <div className="w-px bg-white/10 mx-2 my-2" />
          
          <button 
            onClick={() => onCommand(CommandType.RESET)}
            className="flex items-center gap-2 px-6 py-3 bg-slate-500/20 hover:bg-slate-500/30 text-slate-400 rounded-xl transition-all hover:scale-105 active:scale-95 group font-bold tracking-wide border border-slate-500/30"
          >
            <RotateCcw className="w-5 h-5" />
            PURGE
          </button>
        </div>
      </div>

      {cameraState && (
        <div className="absolute bottom-6 right-6 w-40 h-40 bg-slate-900/60 border border-white/10 shadow-2xl rounded-xl overflow-hidden backdrop-blur-md pointer-events-auto">
          <div 
            className="absolute border-2 border-white/80 bg-white/20 shadow-[0_0_10px_rgba(255,255,255,0.2)]"
            style={{
              left: `${Math.max(0, (cameraState.x / cameraState.worldWidth) * 100)}%`,
              top: `${Math.max(0, (cameraState.y / cameraState.worldHeight) * 100)}%`,
              width: `${Math.min(100, (cameraState.width / cameraState.worldWidth) * 100)}%`,
              height: `${Math.min(100, (cameraState.height / cameraState.worldHeight) * 100)}%`,
            }}
          />
        </div>
      )}
    </div>
  );
};
