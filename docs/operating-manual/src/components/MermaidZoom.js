import { useEffect } from 'react';

export default function MermaidZoom() {
  useEffect(() => {
    // Add zoom and pan functionality to Mermaid diagrams
    const addZoomPan = () => {
      const containers = document.querySelectorAll('.docusaurus-mermaid-container');
      
      containers.forEach(container => {
        const svg = container.querySelector('svg');
        if (!svg || svg.dataset.zoomEnabled) return;
        
        svg.dataset.zoomEnabled = 'true';
        
        let scale = 1;
        let translateX = 0;
        let translateY = 0;
        let isDragging = false;
        let startX, startY;
        
        // Create zoom controls
        const controls = document.createElement('div');
        controls.className = 'mermaid-zoom-controls';
        controls.innerHTML = `
          <button class="zoom-in" title="Zoom In">+</button>
          <button class="zoom-out" title="Zoom Out">−</button>
          <button class="zoom-reset" title="Reset Zoom">⌂</button>
        `;
        controls.style.cssText = `
          position: absolute;
          top: 8px;
          right: 8px;
          display: flex;
          flex-direction: column;
          gap: 2px;
          z-index: 10;
        `;
        
        const buttonStyle = `
          width: 24px;
          height: 24px;
          border: 1px solid #d0d7de;
          background: white;
          border-radius: 3px;
          cursor: pointer;
          font-size: 14px;
          display: flex;
          align-items: center;
          justify-content: center;
          transition: all 0.2s;
        `;
        
        controls.querySelectorAll('button').forEach(btn => {
          btn.style.cssText = buttonStyle;
          btn.addEventListener('mouseenter', () => {
            btn.style.background = '#f6f8fa';
          });
          btn.addEventListener('mouseleave', () => {
            btn.style.background = 'white';
          });
        });
        
        container.appendChild(controls);
        
        const updateTransform = () => {
          svg.style.transform = `translate(${translateX}px, ${translateY}px) scale(${scale})`;
        };
        
        // Zoom controls
        controls.querySelector('.zoom-in').addEventListener('click', () => {
          scale = Math.min(scale * 1.2, 3);
          updateTransform();
        });
        
        controls.querySelector('.zoom-out').addEventListener('click', () => {
          scale = Math.max(scale / 1.2, 0.5);
          updateTransform();
        });
        
        controls.querySelector('.zoom-reset').addEventListener('click', () => {
          scale = 1;
          translateX = 0;
          translateY = 0;
          updateTransform();
        });
        
        // Mouse wheel zoom
        container.addEventListener('wheel', (e) => {
          e.preventDefault();
          const delta = e.deltaY > 0 ? 0.9 : 1.1;
          scale = Math.min(Math.max(scale * delta, 0.5), 3);
          updateTransform();
        });
        
        // Pan functionality
        svg.addEventListener('mousedown', (e) => {
          isDragging = true;
          startX = e.clientX - translateX;
          startY = e.clientY - translateY;
          svg.style.cursor = 'grabbing';
        });
        
        document.addEventListener('mousemove', (e) => {
          if (!isDragging) return;
          translateX = e.clientX - startX;
          translateY = e.clientY - startY;
          updateTransform();
        });
        
        document.addEventListener('mouseup', () => {
          isDragging = false;
          svg.style.cursor = 'grab';
        });
      });
    };
    
    // Run on initial load and when content changes
    addZoomPan();
    
    // Observer for dynamically added content
    const observer = new MutationObserver(addZoomPan);
    observer.observe(document.body, { childList: true, subtree: true });
    
    return () => observer.disconnect();
  }, []);
  
  return null;
}
