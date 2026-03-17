import { Activity } from 'lucide-react';

export const LoadingScreen = () => {
  return (
    <div className="absolute inset-0 z-50 flex items-center justify-center bg-[#1e1b4b]">
      <div className="flex flex-col items-center gap-6">
        <div className="relative">
          <div className="absolute inset-0 bg-indigo-500 rounded-full blur-xl opacity-50 animate-pulse"></div>
          <Activity className="w-16 h-16 text-indigo-400 animate-bounce relative z-10" />
        </div>
        <h2 className="text-2xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-indigo-300 to-purple-400 animate-pulse">
          CONNEXION À LA COLONIE...
        </h2>
        <div className="flex gap-2">
          <div className="w-3 h-3 bg-indigo-500 rounded-full animate-ping" style={{ animationDelay: '0ms' }}></div>
          <div className="w-3 h-3 bg-purple-500 rounded-full animate-ping" style={{ animationDelay: '150ms' }}></div>
          <div className="w-3 h-3 bg-pink-500 rounded-full animate-ping" style={{ animationDelay: '300ms' }}></div>
        </div>
      </div>
    </div>
  );
};
