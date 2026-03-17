import { useEffect, useState, Suspense, lazy } from 'react';
import { apiService } from './services/api';
import { CommandType } from './types';
import type { GameState } from './types';
import { HUD } from './components/HUD';
import { LoadingScreen } from './components/LoadingScreen';

// Load PixiJS asynchronously so HUD and initial HTML load instantly
const SimulationCanvas = lazy(() => import('./components/SimulationCanvas').then(module => ({ default: module.SimulationCanvas })));

export default function App() {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [connected, setConnected] = useState(false);
  const [toast, setToast] = useState<{msg: string, isErr: boolean} | null>(null);
  
  // Responsive stage dimensions
  const [dimensions, setDimensions] = useState({ width: window.innerWidth, height: window.innerHeight });

  useEffect(() => {
    const handleResize = () => {
      setDimensions({ width: window.innerWidth, height: window.innerHeight });
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  useEffect(() => {
    // Start WebSocket
    apiService.connect();

    const unsubscribeState = apiService.onState((state: GameState) => {
      // Debug payload parsing for Task 2
      // console.log("Payload recu:", state);
      setGameState(state);
      setConnected(true);
    });

    const unsubscribeClose = apiService.onClose(() => {
      setConnected(false);
    });

    return () => {
      unsubscribeState();
      unsubscribeClose();
      apiService.disconnect();
    };
  }, []);

  const handleCommand = async (cmd: CommandType) => {
    try {
      await apiService.sendCommand(cmd);
      setToast({ msg: `Commande envoyée avec succès`, isErr: false });
    } catch (e: any) {
      setToast({ msg: `Erreur API: ${e.message}`, isErr: true });
    }
    setTimeout(() => setToast(null), 3000);
  };

  const isSimReady = connected && gameState;

  return (
    <div className="relative w-screen h-screen overflow-hidden bg-[#1e1b4b] text-slate-100 font-sans selection:bg-purple-500/30">
      
      {!isSimReady && <LoadingScreen />}

      {toast && (
        <div className={`absolute top-4 left-1/2 -translate-x-1/2 z-50 px-6 py-3 rounded-full text-sm font-bold shadow-2xl transition-all ${toast.isErr ? 'bg-rose-500/90 text-white border border-rose-400' : 'bg-emerald-500/90 text-white border border-emerald-400'}`}>
          {toast.msg}
        </div>
      )}

      {isSimReady && (
        <>
          {/* PixiJS Canvas Layer */}
          <div className="absolute inset-0 z-0 cursor-crosshair">
            <Suspense fallback={<LoadingScreen />}>
              <SimulationCanvas width={dimensions.width} height={dimensions.height} gameState={gameState} />
            </Suspense>
          </div>

          {/* React UI Overlay Layer */}
          <HUD gameState={gameState} connected={connected} onCommand={handleCommand} />
        </>
      )}

    </div>
  );
}
