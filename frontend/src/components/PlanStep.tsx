import React from 'react';
import { PlanStep } from '../types';

interface PlanStepProps {
  step: PlanStep;
  index: number;
}

const PlanStepComponent: React.FC<PlanStepProps> = ({ step, index }) => {
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'Completed':
        return '✅';
      case 'InProgress':
        return '⏳';
      case 'Failed':
        return '❌';
      default:
        return '⏸️';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Completed':
        return 'from-green-400 to-emerald-500';
      case 'InProgress':
        return 'from-blue-400 to-indigo-500';
      case 'Failed':
        return 'from-red-400 to-pink-500';
      default:
        return 'from-gray-400 to-gray-500';
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'Completed':
        return '已完成';
      case 'InProgress':
        return '执行中';
      case 'Failed':
        return '已失败';
      default:
        return '等待中';
    }
  };

  return (
    <div className="glass rounded-2xl p-6 border border-white/30 backdrop-blur-xl hover:border-white/50 transition-all duration-300 hover:transform hover:scale-[1.02]">
      <div className="flex items-start gap-4">
        {/* Step Number */}
        <div className="flex-shrink-0">
          <div className={`w-12 h-12 rounded-full bg-gradient-to-r ${getStatusColor(step.status)} flex items-center justify-center text-white font-bold text-lg shadow-lg`}>
            {index + 1}
          </div>
        </div>
        
        {/* Step Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-3">
            <h4 className="text-lg font-bold text-white truncate pr-4">
              {step.title}
            </h4>
            <div className={`flex items-center gap-2 px-3 py-1 rounded-full bg-gradient-to-r ${getStatusColor(step.status)} text-white text-sm font-semibold shadow-md`}>
              <span className="text-base">{getStatusIcon(step.status)}</span>
              {getStatusText(step.status)}
            </div>
          </div>
          
          {/* Agent Type */}
          <div className="flex items-center gap-2 mb-3">
            <span className="text-blue-200 text-sm font-medium">🤖 代理类型:</span>
            <span className="bg-blue-500/30 text-blue-100 px-3 py-1 rounded-full text-sm font-semibold border border-blue-400/30">
              {step.agent_type}
            </span>
          </div>
          
          {/* Result */}
          {step.result && (
            <div className="mt-4">
              <div className="bg-white/20 rounded-xl p-4 border border-white/20">
                <h5 className="text-green-200 font-semibold mb-2 flex items-center gap-2">
                  📄 执行结果:
                </h5>
                <p className="text-white/90 text-sm leading-relaxed whitespace-pre-wrap">
                  {step.result}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PlanStepComponent;