'use client';

import { useState } from 'react';
import Router, { Route } from '@/components/Router';

export default function Home() {
  const [currentRoute, setCurrentRoute] = useState<Route>('home');

  const handleRouteChange = (route: Route) => {
    setCurrentRoute(route);
  };

  // 如果不在首页，显示路由页面
  if (currentRoute !== 'home') {
    return <Router currentRoute={currentRoute} onRouteChange={handleRouteChange} />;
  }

  // 首页内容
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-white to-indigo-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900">
      {/* 导航栏 */}
      <nav className="flex items-center justify-between p-6">
        <div className="flex items-center space-x-2">
          <div className="w-8 h-8 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-sm">T</span>
          </div>
          <span className="text-xl font-bold text-gray-900 dark:text-white">ToolAgent</span>
        </div>
        <div className="flex items-center space-x-4">
          <button 
            onClick={() => handleRouteChange('chat')}
            className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
          >
            问答
          </button>
          <button 
            onClick={() => handleRouteChange('ragchat')}
            className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
          >
            RAG
          </button>
          <button 
            onClick={() => handleRouteChange('chrome')}
            className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
          >
            自动化
          </button>
          <button 
            onClick={() => handleRouteChange('knowledge')}
            className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
          >
            知识库
          </button>
          <a 
            href="https://github.com/Aaron-Ben/ToolAgent" 
            target="_blank" 
            rel="noopener noreferrer"
            className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
          >
            GitHub
          </a>
        </div>
      </nav>

      {/* 主要内容区域 */}
      <main className="container mx-auto px-6 py-16">
        {/* 英雄区域 */}
        <div className="text-center mb-16">
          <h1 className="text-5xl md:text-6xl font-bold text-gray-900 dark:text-white mb-6">
            打造智能代理的
            <span className="bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
              AI时代
            </span>
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-300 mb-8 max-w-3xl mx-auto leading-relaxed">
            需要工具，更多的工具。基于 Rust + AI 的智能代理平台，让AI代理帮你完成各种复杂的自动化任务
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <button 
              onClick={() => handleRouteChange('chat')}
              className="px-8 py-3 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-semibold rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 shadow-lg hover:shadow-xl text-center"
            >
              开始使用
            </button>
            <button className="px-8 py-3 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 font-semibold rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
              查看演示
            </button>
          </div>
        </div>

        {/* 功能导航 */}
        <div className="bg-white dark:bg-gray-800 rounded-xl p-8 shadow-lg mb-16">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6 text-center">
            核心功能
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <button 
              onClick={() => handleRouteChange('chat')}
              className="group bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900 dark:to-blue-800 rounded-xl p-6 hover:shadow-lg transition-all duration-200"
            >
              <div className="w-12 h-12 bg-blue-600 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">普通问答</h3>
              <p className="text-sm text-gray-600 dark:text-gray-300">与AI助手进行对话交流</p>
            </button>

            <button 
              onClick={() => handleRouteChange('ragchat')}
              className="group bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900 dark:to-green-800 rounded-xl p-6 hover:shadow-lg transition-all duration-200"
            >
              <div className="w-12 h-12 bg-green-600 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">RAG问答</h3>
              <p className="text-sm text-gray-600 dark:text-gray-300">基于知识库的智能问答</p>
            </button>

            <button 
              onClick={() => handleRouteChange('chrome')}
              className="group bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900 dark:to-purple-800 rounded-xl p-6 hover:shadow-lg transition-all duration-200"
            >
              <div className="w-12 h-12 bg-purple-600 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">浏览器自动化</h3>
              <p className="text-sm text-gray-600 dark:text-gray-300">Chrome自动化任务管理</p>
            </button>

            <button 
              onClick={() => handleRouteChange('knowledge')}
              className="group bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-900 dark:to-orange-800 rounded-xl p-6 hover:shadow-lg transition-all duration-200"
            >
              <div className="w-12 h-12 bg-orange-600 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">知识库管理</h3>
              <p className="text-sm text-gray-600 dark:text-gray-300">管理文档知识库</p>
            </button>
          </div>
        </div>

      </main>
    </div>
  );
}