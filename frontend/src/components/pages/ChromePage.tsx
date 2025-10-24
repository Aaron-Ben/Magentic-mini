'use client';

import { useState, useEffect } from 'react';

interface Task {
  id: string;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  description: string;
  createdAt: Date;
  completedAt?: Date;
}

interface ChromePageProps {
  onBack: () => void;
}

export default function ChromePage({ onBack }: ChromePageProps) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [isClient, setIsClient] = useState(false);

  useEffect(() => {
    setIsClient(true);
    setTasks([
      {
        id: '1',
        name: '网页截图任务',
        status: 'completed',
        description: '对指定网页进行截图并保存',
        createdAt: new Date(Date.now() - 3600000),
        completedAt: new Date(Date.now() - 3000000),
      },
      {
        id: '2',
        name: '表单填写任务',
        status: 'running',
        description: '自动填写网页表单并提交',
        createdAt: new Date(Date.now() - 1800000),
      },
    ]);
  }, []);
  const [newTaskName, setNewTaskName] = useState('');
  const [newTaskDescription, setNewTaskDescription] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  const handleCreateTask = async () => {
    if (!newTaskName.trim() || isCreating) return;

    setIsCreating(true);
    
    const newTask: Task = {
      id: Date.now().toString(),
      name: newTaskName,
      status: 'pending',
      description: newTaskDescription,
      createdAt: new Date(),
    };

    setTasks(prev => [newTask, ...prev]);
    setNewTaskName('');
    setNewTaskDescription('');
    setIsCreating(false);
  };

  const handleStartTask = async (taskId: string) => {
    setTasks(prev => prev.map(task => 
      task.id === taskId 
        ? { ...task, status: 'running' as const }
        : task
    ));

    // 模拟任务执行
    setTimeout(() => {
      setTasks(prev => prev.map(task => 
        task.id === taskId 
          ? { 
              ...task, 
              status: Math.random() > 0.2 ? 'completed' as const : 'failed' as const,
              completedAt: new Date()
            }
          : task
      ));
    }, 3000);
  };

  const getStatusColor = (status: Task['status']) => {
    switch (status) {
      case 'pending': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
      case 'running': return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
      case 'completed': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'failed': return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
    }
  };

  const getStatusText = (status: Task['status']) => {
    switch (status) {
      case 'pending': return '等待中';
      case 'running': return '执行中';
      case 'completed': return '已完成';
      case 'failed': return '失败';
      default: return '未知';
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

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">浏览器自动化</h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">使用 Chrome 浏览器进行自动化任务管理</p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 创建任务 */}
          <div className="lg:col-span-1">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">创建新任务</h2>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    任务名称
                  </label>
                  <input
                    type="text"
                    value={newTaskName}
                    onChange={(e) => setNewTaskName(e.target.value)}
                    className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                    placeholder="输入任务名称"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    任务描述
                  </label>
                  <textarea
                    value={newTaskDescription}
                    onChange={(e) => setNewTaskDescription(e.target.value)}
                    className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                    rows={3}
                    placeholder="描述任务内容"
                  />
                </div>
                <button
                  onClick={handleCreateTask}
                  disabled={!newTaskName.trim() || isCreating}
                  className="w-full bg-blue-600 text-white py-2 px-4 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  {isCreating ? '创建中...' : '创建任务'}
                </button>
              </div>
            </div>
          </div>

          {/* 任务列表 */}
          <div className="lg:col-span-2">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">任务列表</h2>
              <div className="space-y-4">
                {tasks.length === 0 ? (
                  <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                    暂无任务
                  </div>
                ) : (
                  tasks.map((task) => (
                    <div key={task.id} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <h3 className="font-medium text-gray-900 dark:text-white">{task.name}</h3>
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(task.status)}`}>
                          {getStatusText(task.status)}
                        </span>
                      </div>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">{task.description}</p>
                      <div className="flex items-center justify-between">
                        <div className="text-xs text-gray-500 dark:text-gray-400">
                          创建时间: {isClient ? task.createdAt.toLocaleString() : ''}
                          {task.completedAt && (
                            <span className="ml-4">
                              完成时间: {isClient ? task.completedAt.toLocaleString() : ''}
                            </span>
                          )}
                        </div>
                        <div className="flex space-x-2">
                          {task.status === 'pending' && (
                            <button
                              onClick={() => handleStartTask(task.id)}
                              className="px-3 py-1 bg-green-600 text-white text-xs rounded hover:bg-green-700 transition-colors"
                            >
                              开始执行
                            </button>
                          )}
                          {task.status === 'running' && (
                            <div className="flex items-center space-x-1 text-blue-600 dark:text-blue-400">
                              <div className="w-2 h-2 bg-blue-600 rounded-full animate-pulse"></div>
                              <span className="text-xs">执行中...</span>
                            </div>
                          )}
                          <button className="px-3 py-1 bg-gray-600 text-white text-xs rounded hover:bg-gray-700 transition-colors">
                            查看详情
                          </button>
                        </div>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
