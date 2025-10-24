'use client';

import { useState, useEffect } from 'react';

interface Document {
  id: string;
  name: string;
  type: 'pdf' | 'txt' | 'md' | 'docx';
  size: string;
  uploadDate: Date;
  status: 'processing' | 'ready' | 'error';
}

interface KnowledgePageProps {
  onBack: () => void;
}

export default function KnowledgePage({ onBack }: KnowledgePageProps) {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [isClient, setIsClient] = useState(false);

  useEffect(() => {
    setIsClient(true);
    setDocuments([
      {
        id: '1',
        name: '产品手册.pdf',
        type: 'pdf',
        size: '2.3 MB',
        uploadDate: new Date(Date.now() - 86400000),
        status: 'ready',
      },
      {
        id: '2',
        name: '技术文档.md',
        type: 'md',
        size: '156 KB',
        uploadDate: new Date(Date.now() - 172800000),
        status: 'ready',
      },
      {
        id: '3',
        name: 'FAQ.txt',
        type: 'txt',
        size: '89 KB',
        uploadDate: new Date(Date.now() - 259200000),
        status: 'processing',
      },
    ]);
  }, []);
  const [isUploading, setIsUploading] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<FileList | null>(null);

  const handleFileUpload = async () => {
    if (!selectedFiles || selectedFiles.length === 0) return;

    setIsUploading(true);

    // 模拟文件上传
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

      setDocuments(prev => [newDocument, ...prev]);

      // 模拟处理完成
      setTimeout(() => {
        setDocuments(prev => prev.map(doc => 
          doc.id === newDocument.id 
            ? { ...doc, status: 'ready' as const }
            : doc
        ));
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
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">知识库管理</h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">管理你的文档知识库，为 RAG 问答提供数据支持</p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 上传区域 */}
          <div className="lg:col-span-1">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">上传文档</h2>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    选择文件
                  </label>
                  <input
                    type="file"
                    multiple
                    accept=".pdf,.txt,.md,.docx"
                    onChange={handleFileSelect}
                    className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    支持 PDF、TXT、MD、DOCX 格式
                  </p>
                </div>
                {selectedFiles && selectedFiles.length > 0 && (
                  <div className="text-sm text-gray-600 dark:text-gray-400">
                    已选择 {selectedFiles.length} 个文件
                  </div>
                )}
                <button
                  onClick={handleFileUpload}
                  disabled={!selectedFiles || selectedFiles.length === 0 || isUploading}
                  className="w-full bg-blue-600 text-white py-2 px-4 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  {isUploading ? '上传中...' : '上传文档'}
                </button>
              </div>
            </div>

            {/* 统计信息 */}
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 mt-6">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">统计信息</h3>
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">总文档数</span>
                  <span className="font-medium text-gray-900 dark:text-white">{documents.length}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">已就绪</span>
                  <span className="font-medium text-green-600 dark:text-green-400">
                    {documents.filter(doc => doc.status === 'ready').length}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">处理中</span>
                  <span className="font-medium text-yellow-600 dark:text-yellow-400">
                    {documents.filter(doc => doc.status === 'processing').length}
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* 文档列表 */}
          <div className="lg:col-span-2">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">文档列表</h2>
              <div className="space-y-4">
                {documents.length === 0 ? (
                  <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                    暂无文档，请上传文档到知识库
                  </div>
                ) : (
                  documents.map((doc) => (
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
                          <button className="px-3 py-1 bg-gray-600 text-white text-xs rounded hover:bg-gray-700 transition-colors">
                            删除
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
