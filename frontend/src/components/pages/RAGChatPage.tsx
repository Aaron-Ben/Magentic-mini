'use client';

import { useState, useEffect } from 'react';

interface Message {
  id: string;
  content: string;
  role: 'user' | 'assistant';
  timestamp: Date;
  sources?: string[];
}

interface RAGChatPageProps {
  onBack: () => void;
}

export default function RAGChatPage({ onBack }: RAGChatPageProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isClient, setIsClient] = useState(false);

  useEffect(() => {
    setIsClient(true);
    setMessages([
      {
        id: '1',
        content: '你好！我是 RAG 智能助手，可以基于知识库回答你的问题。请告诉我你想了解什么？',
        role: 'assistant',
        timestamp: new Date(),
      },
    ]);
  }, []);
  const [inputMessage, setInputMessage] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleSendMessage = async () => {
    if (!inputMessage.trim() || isLoading) return;

    const userMessage: Message = {
      id: Date.now().toString(),
      content: inputMessage,
      role: 'user',
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInputMessage('');
    setIsLoading(true);

    try {
      // 模拟RAG API调用
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        content: `基于知识库，我为你找到了相关信息："${inputMessage}"。这是通过 RAG 技术检索到的答案。`,
        role: 'assistant',
        timestamp: new Date(),
        sources: [
          '知识库文档1.pdf',
          '技术文档2.md',
          'FAQ文档3.txt'
        ],
      };

      setMessages(prev => [...prev, assistantMessage]);
    } catch (error) {
      console.error('RAG查询失败:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* 导航栏 */}
      <nav className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <div className="flex items-center space-x-2">
                <div className="w-8 h-8 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-lg flex items-center justify-center">
                  <span className="text-white font-bold text-sm">T</span>
                </div>
                <span className="text-xl font-bold text-gray-900 dark:text-white">ToolAgent</span>
              </div>
            </div>
            <div className="flex items-center space-x-4">
              <button 
                onClick={onBack}
                className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
              >
                返回首页
              </button>
            </div>
          </div>
        </div>
      </nav>

      {/* RAG聊天界面 */}
      <div className="max-w-4xl mx-auto px-4 py-6">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg h-[calc(100vh-200px)] flex flex-col">
          {/* 聊天标题 */}
          <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-lg font-semibold text-gray-900 dark:text-white">RAG 问答</h1>
                <p className="text-sm text-gray-600 dark:text-gray-400">基于知识库的智能问答</p>
              </div>
              <div className="flex items-center space-x-2">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <span className="text-xs text-gray-500 dark:text-gray-400">知识库已连接</span>
              </div>
            </div>
          </div>

          {/* 消息列表 */}
          <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
            {messages.map((message) => (
              <div
                key={message.id}
                className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
              >
                <div
                  className={`max-w-xs lg:max-w-md px-4 py-2 rounded-lg ${
                    message.role === 'user'
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white'
                  }`}
                >
                  <p className="text-sm">{message.content}</p>
                  <p className="text-xs mt-1 opacity-70">
                    {isClient ? message.timestamp.toLocaleTimeString() : ''}
                  </p>
                  {message.sources && message.sources.length > 0 && (
                    <div className="mt-2 pt-2 border-t border-gray-300 dark:border-gray-600">
                      <p className="text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">参考来源:</p>
                      <div className="space-y-1">
                        {message.sources.map((source, index) => (
                          <div key={index} className="text-xs text-blue-600 dark:text-blue-400 hover:underline cursor-pointer">
                            📄 {source}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            ))}
            {isLoading && (
              <div className="flex justify-start">
                <div className="bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white px-4 py-2 rounded-lg">
                  <div className="flex items-center space-x-2">
                    <div className="flex space-x-1">
                      <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></div>
                      <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{animationDelay: '0.1s'}}></div>
                      <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{animationDelay: '0.2s'}}></div>
                    </div>
                    <span className="text-xs">正在检索知识库...</span>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* 输入区域 */}
          <div className="px-6 py-4 border-t border-gray-200 dark:border-gray-700">
            <div className="flex space-x-4">
              <textarea
                value={inputMessage}
                onChange={(e) => setInputMessage(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="输入你的问题，我将从知识库中查找相关信息..."
                className="flex-1 resize-none border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                rows={2}
                disabled={isLoading}
              />
              <button
                onClick={handleSendMessage}
                disabled={!inputMessage.trim() || isLoading}
                className="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                查询
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
