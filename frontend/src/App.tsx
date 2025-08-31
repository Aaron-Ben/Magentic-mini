import React, { useState, useEffect } from 'react';
import { apiService } from './api';
import { Plan } from './types';
import PlanStepComponent from './components/PlanStep';

// 全局样式组件
const GlobalStyles = () => {
  useEffect(() => {
    // 应用全局样式到body
    const body = document.body;
    body.style.margin = '0';
    body.style.fontFamily = `-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif`;
    (body.style as any).webkitFontSmoothing = 'antialiased';
    (body.style as any).mozOsxFontSmoothing = 'grayscale';
    body.style.minHeight = '100vh';
    
    // 应用样式到root元素
    const root = document.getElementById('root');
    if (root) {
      root.style.minHeight = '100vh';
    }
    
    // 添加滚动条样式
    const style = document.createElement('style');
    style.textContent = `
      ::-webkit-scrollbar {
        width: 8px;
      }
      
      ::-webkit-scrollbar-track {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 4px;
      }
      
      ::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.3);
        border-radius: 4px;
      }
      
      ::-webkit-scrollbar-thumb:hover {
        background: rgba(255, 255, 255, 0.5);
      }
      
      .glass {
        backdrop-filter: blur(16px);
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
      }
      
      .text-shadow {
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
      }
    `;
    document.head.appendChild(style);
    
    return () => {
      document.head.removeChild(style);
    };
  }, []);
  
  return null;
};

function App() {
  const [userInput, setUserInput] = useState('');
  const [currentPlan, setCurrentPlan] = useState<Plan | null>(null);
  const [loading, setLoading] = useState(false);
  const [executing, setExecuting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!userInput.trim()) return;

    setLoading(true);
    setError(null);
    
    try {
      const response = await apiService.createPlan(userInput.trim());
      setCurrentPlan(response.plan);
    } catch (err: any) {
      setError(err.response?.data?.error || '生成计划时出现错误');
    } finally {
      setLoading(false);
    }
  };

  const handleExecute = async () => {
    if (!currentPlan) return;

    setExecuting(true);
    setError(null);

    try {
      const response = await apiService.executePlan(currentPlan.id);
      setCurrentPlan(response.plan);
    } catch (err: any) {
      setError(err.response?.data?.error || '执行计划时出现错误');
    } finally {
      setExecuting(false);
    }
  };

  const handleClear = () => {
    setCurrentPlan(null);
    setUserInput('');
    setError(null);
  };

  return (
    <>
      <GlobalStyles />
      <div className="min-h-screen bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800 p-4">
      <div className="max-w-5xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <div className="mb-6 animate-bounce">
            <span className="text-7xl drop-shadow-lg">🔮</span>
          </div>
          <h1 className="text-5xl font-bold text-white mb-4 text-shadow tracking-tight">
            Mini Magentic-UI
          </h1>
          <p className="text-xl text-white/90 font-medium max-w-md mx-auto leading-relaxed">
            ✨ 智能任务规划和执行系统 ✨
          </p>
        </div>

        {/* Input Form */}
        <div className="glass rounded-3xl shadow-2xl p-8 mb-8 border border-white/20 backdrop-blur-xl">
          <form onSubmit={handleSubmit}>
            <div className="mb-8">
              <label className="block text-xl font-bold text-white mb-4 flex items-center gap-2">
                💭 告诉我你想要做什么：
              </label>
              <textarea
                value={userInput}
                onChange={(e) => setUserInput(e.target.value)}
                placeholder="例如：我想学习 React，我想制作一个网站，我想分析数据...\n\n💡 提示：描述得越详细，我就能为你制定越精准的计划！"
                className="w-full h-40 p-6 border-2 border-white/30 rounded-2xl text-base font-medium bg-white/90 focus:bg-white focus:border-blue-400 focus:ring-4 focus:ring-blue-200/50 transition-all duration-300 resize-none placeholder-gray-500 shadow-inner"
              />
            </div>
            <div className="flex gap-4 justify-end">
              {currentPlan && (
                <button
                  type="button"
                  onClick={handleClear}
                  className="px-8 py-4 bg-gradient-to-r from-gray-500 to-gray-600 hover:from-gray-600 hover:to-gray-700 text-white font-bold rounded-2xl transition-all duration-300 shadow-lg hover:shadow-2xl transform hover:-translate-y-1 hover:scale-105 flex items-center gap-2"
                >
                  🗑️ 清除
                </button>
              )}
              <button
                type="submit"
                disabled={loading || !userInput.trim()}
                className={`px-10 py-4 font-bold rounded-2xl transition-all duration-300 shadow-lg hover:shadow-2xl transform flex items-center gap-2 ${
                  loading || !userInput.trim()
                    ? 'bg-gray-400 cursor-not-allowed'
                    : 'bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 hover:-translate-y-1 hover:scale-105'
                } text-white`}
              >
                {loading ? (
                  <>
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                    生成中...
                  </>
                ) : (
                  <>
                    ✨ 生成计划
                  </>
                )}
              </button>
            </div>
          </form>
        </div>

        {/* Error Message */}
        {error && (
          <div className="glass border-l-4 border-red-400 p-6 mb-8 rounded-2xl shadow-xl backdrop-blur-xl animate-pulse">
            <div className="flex items-center">
              <span className="text-red-400 text-2xl mr-4">❌</span>
              <div>
                <h3 className="text-red-100 font-bold text-lg">错误</h3>
                <p className="text-red-200 mt-2 leading-relaxed">{error}</p>
              </div>
            </div>
          </div>
        )}

        {/* Plan Display */}
        {currentPlan && (
          <div className="glass rounded-3xl shadow-2xl p-8 border border-white/20 backdrop-blur-xl">
            <div className="flex flex-col lg:flex-row lg:justify-between lg:items-start mb-10 gap-6">
              <div className="flex-1">
                <h2 className="text-3xl font-bold text-white mb-4 flex items-center gap-3 text-shadow">
                  📋 执行计划
                </h2>
                <div className="bg-white/20 rounded-2xl p-6 backdrop-blur-sm">
                  <p className="text-lg text-white font-medium leading-relaxed">
                    {currentPlan.task}
                  </p>
                </div>
              </div>
              <button
                onClick={handleExecute}
                disabled={executing}
                className={`px-10 py-4 font-bold rounded-2xl transition-all duration-300 shadow-lg flex items-center gap-3 ${
                  executing
                    ? 'bg-gray-500 cursor-not-allowed'
                    : 'bg-gradient-to-r from-green-500 to-emerald-600 hover:from-green-600 hover:to-emerald-700 hover:shadow-2xl transform hover:-translate-y-1 hover:scale-105'
                } text-white`}
              >
                {executing ? (
                  <>
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                    执行中...
                  </>
                ) : (
                  <>
                    🚀 执行计划
                  </>
                )}
              </button>
            </div>

            {/* Plan Steps */}
            <div>
              <h3 className="text-2xl font-bold text-white mb-8 flex items-center gap-3 text-shadow">
                🔧 执行步骤 
                <span className="bg-gradient-to-r from-blue-400 to-purple-500 text-white px-4 py-2 rounded-full text-base font-bold shadow-lg">
                  {currentPlan.steps.length}
                </span>
              </h3>
              <div className="space-y-6">
                {currentPlan.steps.map((step, index) => (
                  <PlanStepComponent key={step.id} step={step} index={index} />
                ))}
              </div>
            </div>
          </div>
        )}

        {/* Empty State */}
        {!currentPlan && !loading && (
          <div className="glass rounded-3xl p-16 text-center border border-white/20 backdrop-blur-xl">
            <div className="mb-8 animate-pulse">
              <span className="text-8xl drop-shadow-lg">🤖</span>
            </div>
            <h3 className="text-3xl font-bold text-white mb-6 text-shadow">
              准备好开始了吗？
            </h3>
            <p className="text-xl text-white/90 leading-relaxed max-w-lg mx-auto mb-10">
              在上方输入框中描述你想要完成的任务，我会为你生成详细的执行计划。
            </p>
            <div className="flex flex-wrap justify-center gap-4 text-sm">
              <span className="bg-white/20 text-white px-4 py-2 rounded-full font-medium">💡 学习新技能</span>
              <span className="bg-white/20 text-white px-4 py-2 rounded-full font-medium">🌐 制作网站</span>
              <span className="bg-white/20 text-white px-4 py-2 rounded-full font-medium">📊 数据分析</span>
              <span className="bg-white/20 text-white px-4 py-2 rounded-full font-medium">🔧 自动化任务</span>
            </div>
          </div>
        )}  
      </div>
      </div>
    </>
  );
}

export default App;