'use client';

import { useState } from 'react';
import ChatPage from '@/components/pages/ChatPage';
import RAGChatPage from '@/components/pages/RAGChatPage';
import ChromePage from '@/components/pages/ChromePage';
import KnowledgePage from '@/components/pages/KnowledgePage';

type Route = 'home' | 'chat' | 'ragchat' | 'chrome' | 'knowledge';

interface RouterProps {
  currentRoute: Route;
  onRouteChange: (route: Route) => void;
}

export default function Router({ currentRoute, onRouteChange }: RouterProps) {
  const renderPage = () => {
    switch (currentRoute) {
      case 'chat':
        return <ChatPage onBack={() => onRouteChange('home')} />;
      case 'ragchat':
        return <RAGChatPage onBack={() => onRouteChange('home')} />;
      case 'chrome':
        return <ChromePage onBack={() => onRouteChange('home')} />;
      case 'knowledge':
        return <KnowledgePage onBack={() => onRouteChange('home')} />;
      case 'home':
      default:
        return null;
    }
  };

  return renderPage();
}

export type { Route };
