import React from 'react';
import MermaidZoom from '../components/MermaidZoom';

// Default implementation, that you can customize
export default function Root({children}) {
  return (
    <>
      {children}
      <MermaidZoom />
    </>
  );
}
