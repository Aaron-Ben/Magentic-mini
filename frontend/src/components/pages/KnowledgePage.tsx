'use client';

import { useState, useEffect } from 'react';

// === 类型定义 ===
interface Document {
  id: string;
  name: string;
  type: 'pdf' | 'txt' | 'md' | 'docx';
  size: string;
  uploadDate: Date;
  status: 'processing' | 'ready' | 'error';
}

interface KnowledgeBase {
  id: string;
  name: string;
  description: string;
  model: string;
  chunkSize: number;
  overlapSize: number;
  createdAt: Date;
  documents: Document[]; // 此知识库下的文档
}

interface KnowledgePageProps {
  onBack: () => void;
}

export default function KnowledgeManagementPage({ onBack }: KnowledgePageProps) {
  const [isClient, setIsClient] = useState(false);
  const [knowledgeBases, setKnowledgeBases] = useState<KnowledgeBase[]>([]);
  const [selectedKB, setSelectedKB] = useState<KnowledgeBase | null>(null); // 当前选中的知识库
  const [isUploading, setIsUploading] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<FileList | null>(null);
  const [showNewKBModal, setShowNewKBModal] = useState(false); // 是否显示新建弹窗
  const [newKBName, setNewKBName] = useState('');
  const [newKBDescription, setNewKBDescription] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<{ type: 'kb' | 'doc', id: string } | null>(null);

  useEffect(() => {
    setIsClient(true);
  }, []);

  // 新建知识库
  const handleCreateKB = () => {
    if (!newKBName.trim()) return;

    const newKB: KnowledgeBase = {
      id: Date.now().toString(),
      name: newKBName,
      description: newKBDescription,
      model: 'text-embedding-ada-002', // 默认模型
      chunkSize: 1000,
      overlapSize: 200,
      createdAt: new Date(),
      documents: [],
    };

    setKnowledgeBases(prev => [...prev, newKB]);
    setNewKBName('');
    setNewKBDescription('');
    setShowNewKBModal(false);
    setSelectedKB(newKB); // 自动选中新创建的知识库
  };

  // 选择知识库
  const handleSelectKB = (kb: KnowledgeBase) => {
    setSelectedKB(kb);
  };

  // 上传文件到当前选中知识库
  const handleFileUpload = async () => {
    if (!selectedKB || !selectedFiles || selectedFiles.length === 0) return;

    setIsUploading(true);

    for (let i = 0; i < selectedFiles.length; i++) {
      const file = selectedFiles[i];
      const newDocument: Document = {
        id: Date.now().toString() + i,
        name: file.name,
        type: file.name.split('.').pop() as Document['type'],
        size: (file.size / 1024 / 1024).toFixed(2) + ' MB',
        uploadDate: new Date(),
        status: 'processing',
      };

      // 更新选中知识库的文档列表
      setKnowledgeBases(prev =>
        prev.map(kb =>
          kb.id === selectedKB.id
            ? {
                ...kb,
                documents: [newDocument, ...kb.documents],
              }
            : kb
        )
      );

      // 模拟处理完成
      setTimeout(() => {
        setKnowledgeBases(prev =>
          prev.map(kb =>
            kb.id === selectedKB.id
              ? {
                  ...kb,
                  documents: kb.documents.map(doc =>
                    doc.id === newDocument.id
                      ? { ...doc, status: 'ready' as const }
                      : doc
                  ),
                }
              : kb
          )
        );
      }, 2000 + Math.random() * 3000);
    }

    setSelectedFiles(null);
    setIsUploading(false);
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSelectedFiles(e.target.files);
  };

  const getStatusColor = (status: Document['status']) => {
    switch (status) {
      case 'processing': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
      case 'ready': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'error': return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
    }
  };

  const getStatusText = (status: Document['status']) => {
    switch (status) {
      case 'processing': return '处理中';
      case 'ready': return '已就绪';
      case 'error': return '错误';
      default: return '未知';
    }
  };

  const getFileIcon = (type: Document['type']) => {
    switch (type) {
      case 'pdf': return '📄';
      case 'txt': return '📝';
      case 'md': return '📋';
      case 'docx': return '📊';
      default: return '📁';
    }
  };

  // 删除确认逻辑
  const handleConfirmDelete = () => {
    if (!deleteTarget) return;

    if (deleteTarget.type === 'kb') {
      // 删除知识库
      setKnowledgeBases(prev => prev.filter(kb => kb.id !== deleteTarget.id));
      if (selectedKB?.id === deleteTarget.id) {
        setSelectedKB(null);
      }
    } else {
      // 删除文档
      if (!selectedKB) return;
      setKnowledgeBases(prev =>
        prev.map(kb =>
          kb.id === selectedKB.id
            ? {
                ...kb,
                documents: kb.documents.filter(doc => doc.id !== deleteTarget.id),
              }
            : kb
        )
      );
    }

    setDeleteTarget(null); // 关闭弹窗
  };

  // 刷新当前知识库（模拟）
  const handleRefresh = () => {
    // 实际项目中可重新拉取数据
    console.log("刷新知识库:", selectedKB?.name);
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
        {/* 页面标题 + 操作区 */}
        <div className="flex justify-between items-center mb-6">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white">知识库管理</h1>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              共 {knowledgeBases.length} 个知识库 · {knowledgeBases.reduce((sum, kb) => sum + kb.documents.length, 0)} 个文档
            </p>
          </div>
          <div className="flex space-x-2">
            <button
              onClick={() => setShowNewKBModal(true)}
              className="bg-indigo-600 text-white px-4 py-2 rounded-lg hover:bg-indigo-700 transition-colors"
            >
              + 新建知识库
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
          {/* 左侧边栏 - 知识库列表 */}
          <div className="lg:col-span-1">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">知识库列表</h2>
              <div className="space-y-2 max-h-[500px] overflow-y-auto pr-2">
                {knowledgeBases.length === 0 ? (
                  <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                    <div className="mb-2">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-12 w-12 mx-auto text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9M5 11V9m2 2a2 2 0 100 4h12a2 2 0 100-4H7z" />
                      </svg>
                    </div>
                    暂无知识库
                  </div>
                ) : (
                  knowledgeBases.map((kb) => (
                    <div
                      key={kb.id}
                      onClick={() => handleSelectKB(kb)}
                      className={`p-3 rounded-lg cursor-pointer transition-colors ${
                        selectedKB?.id === kb.id
                          ? 'bg-blue-100 dark:bg-blue-900 border border-blue-500'
                          : 'hover:bg-gray-100 dark:hover:bg-gray-700'
                      }`}
                    >
                      <div className="flex justify-between items-center">
                        <div>
                          <h3 className="font-medium text-gray-900 dark:text-white">{kb.name}</h3>
                          <p className="text-xs text-gray-500 dark:text-gray-400 truncate">{kb.description}</p>
                        </div>
                      </div>
                      <div className="flex justify-between items-center mt-2 text-xs text-gray-500 dark:text-gray-400">
                        <span>{kb.documents.length} 个文档</span>
                        <span>{isClient ? kb.createdAt.toLocaleDateString() : ''}</span>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>

          {/* 右侧主内容区 - 知识库详情 & 文档列表 */}
          <div className="lg:col-span-3">
            {selectedKB ? (
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
                {/* 知识库头部信息 */}
                <div className="flex justify-between items-start mb-4">
                  <div>
                    <h2 className="text-xl font-bold text-gray-900 dark:text-white">{selectedKB.name}</h2>
                    <p className="text-gray-600 dark:text-gray-400">{selectedKB.description}</p>
                  </div>
                  <div className="flex space-x-2">
                    <button className="px-3 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 transition-colors">
                      编辑
                    </button>
                    <button
                      onClick={() => setDeleteTarget({ type: 'kb', id: selectedKB.id })}
                      className="px-3 py-1 bg-gray-600 text-white text-xs rounded hover:bg-gray-700 transition-colors"
                    >
                      删除
                    </button>
                  </div>
                </div>

                {/* 知识库元信息 */}
                <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-4">
                  <div className="flex flex-wrap gap-4 text-sm text-gray-600 dark:text-gray-400">
                    <div>模型: <span className="font-medium">{selectedKB.model}</span></div>
                    <div>分块大小: <span className="font-medium">{selectedKB.chunkSize}</span></div>
                    <div>重叠大小: <span className="font-medium">{selectedKB.overlapSize}</span></div>
                    <div>创建时间: <span className="font-medium">{isClient ? selectedKB.createdAt.toLocaleString() : ''}</span></div>
                  </div>
                </div>

                {/* 文档操作按钮 */}
                <div className="flex justify-between items-center mb-4">
                  <div className="flex space-x-2">
                    <button
                      onClick={() => document.getElementById('fileInput')?.click()}
                      className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                      上传文档
                    </button>
                    <input
                      id="fileInput"
                      type="file"
                      multiple
                      accept=".pdf,.txt,.md,.docx"
                      onChange={handleFileSelect}
                      className="hidden"
                    />
                    <button
                      onClick={handleFileUpload}
                      disabled={!selectedFiles || selectedFiles.length === 0 || isUploading}
                      className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    >
                      {isUploading ? '上传中...' : '确认上传'}
                    </button>
                  </div>
                  <button
                    onClick={handleRefresh}
                    className="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                  >
                    刷新
                  </button>
                </div>

                {/* 文档列表 */}
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">文档列表</h3>
                {selectedKB.documents.length === 0 ? (
                  <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                    <div className="mb-2">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-12 w-12 mx-auto text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9M5 11V9m2 2a2 2 0 100 4h12a2 2 0 100-4H7z" />
                      </svg>
                    </div>
                    暂无文档，请上传文档
                  </div>
                ) : (
                  <div className="space-y-4 max-h-[500px] overflow-y-auto pr-2">
                    {selectedKB.documents.map((doc) => (
                      <div key={doc.id} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                        <div className="flex items-center justify-between mb-2">
                          <div className="flex items-center space-x-3">
                            <span className="text-2xl">{getFileIcon(doc.type)}</span>
                            <div>
                              <h3 className="font-medium text-gray-900 dark:text-white">{doc.name}</h3>
                              <p className="text-sm text-gray-600 dark:text-gray-400">
                                {doc.type.toUpperCase()} • {doc.size}
                              </p>
                            </div>
                          </div>
                          <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(doc.status)}`}>
                            {getStatusText(doc.status)}
                          </span>
                        </div>
                        <div className="flex items-center justify-between">
                          <div className="text-xs text-gray-500 dark:text-gray-400">
                            上传时间: {isClient ? doc.uploadDate.toLocaleString() : ''}
                          </div>
                          <div className="flex space-x-2">
                            <button className="px-3 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 transition-colors">
                              预览
                            </button>
                            <button
                              onClick={() => setDeleteTarget({type: 'doc', id: doc.id})}
                              className="px-3 py-1 bg-gray-600 text-white text-xs rounded hover:bg-gray-700 transition-colors"
                            >
                              删除
                            </button>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ) : (
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 text-center">
                <div className="mb-4">
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-12 w-12 mx-auto text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9M3 7l6 6m0 0l6-6m-6 6h12" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">选择知识库</h3>
                <p className="text-gray-600 dark:text-gray-400">
                  请从左侧列表中选择一个知识库来查看详情和管理文档
                </p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 新建知识库弹窗 */}
      {showNewKBModal && (
        <div
          className="fixed inset-0 flex items-center justify-center z-50 bg-black/10 backdrop-blur-sm"
          onClick={(e) => {
            // 点击弹窗外部关闭
            if (e.target === e.currentTarget) {
              setShowNewKBModal(false);
            }
          }}
        >
          <div className="relative bg-white dark:bg-gray-800 rounded-xl shadow-2xl p-6 w-full max-w-md mx-4 border border-gray-200 dark:border-gray-700">
            {/* 关闭按钮（可选但推荐） */}
            <button
              onClick={() => setShowNewKBModal(false)}
              className="absolute top-4 right-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
              aria-label="关闭"
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
            </button>

            <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-4 text-center">
              新建知识库
            </h2>
            <div className="space-y-4 mt-2">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  名称 *
                </label>
                <input
                  type="text"
                  value={newKBName}
                  onChange={(e) => setNewKBName(e.target.value)}
                  className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                  placeholder="请输入知识库名称"
                  autoFocus // 自动聚焦到输入框
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  描述
                </label>
                <textarea
                  value={newKBDescription}
                  onChange={(e) => setNewKBDescription(e.target.value)}
                  className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                  rows={3}
                  placeholder="请输入知识库描述"
                />
              </div>
            </div>
            <div className="flex justify-center space-x-3 mt-6">
              <button
                onClick={() => setShowNewKBModal(false)}
                className="px-5 py-2.5 text-gray-700 dark:text-gray-300 font-medium rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleCreateKB}
                disabled={!newKBName.trim()}
                className="px-5 py-2.5 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors shadow-sm"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 删除确认弹窗 */}
      {deleteTarget && (
        <div
          className="fixed inset-0 flex items-center justify-center z-50 bg-black/10 backdrop-blur-sm"
          onClick={(e) => {
            // 点击弹窗外部关闭
            if (e.target === e.currentTarget) {
              setDeleteTarget(null);
            }
          }}
        >
          <div className="relative bg-white dark:bg-gray-800 rounded-xl shadow-2xl p-6 w-full max-w-md mx-4 border border-gray-200 dark:border-gray-700">
            {/* 关闭按钮（可选） */}
            <button
              onClick={() => setDeleteTarget(null)}
              className="absolute top-4 right-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
              aria-label="关闭"
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
            </button>

            <div className="pt-2">
              <div className="w-12 h-12 bg-red-100 dark:bg-red-900/50 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6 text-red-600 dark:text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              </div>
              <h2 className="text-xl font-bold text-center text-gray-900 dark:text-white mb-2">
                确认删除
              </h2>
              <p className="text-gray-600 dark:text-gray-400 text-center mb-6 px-2">
                {deleteTarget.type === 'kb'
                  ? '此操作将永久删除知识库及其所有文档，无法恢复。'
                  : '此文档将被永久删除，无法恢复。'}
              </p>
              <div className="flex justify-center space-x-3">
                <button
                  onClick={() => setDeleteTarget(null)}
                  className="px-5 py-2.5 text-gray-700 dark:text-gray-300 font-medium rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={handleConfirmDelete}
                  className="px-5 py-2.5 bg-red-600 text-white font-medium rounded-lg hover:bg-red-700 transition-colors shadow-sm"
                >
                  删除
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}