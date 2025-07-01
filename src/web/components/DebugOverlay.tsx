import React, { useState, useEffect } from 'react';

interface Measurements {
  screen: {
    width: number;
    height: number;
    availWidth: number;
    availHeight: number;
    colorDepth: number;
    pixelDepth: number;
  };
  window: {
    innerWidth: number;
    innerHeight: number;
    outerWidth: number;
    outerHeight: number;
    devicePixelRatio: number;
    scrollX: number;
    scrollY: number;
    screenX: number;
    screenY: number;
  };
  document: {
    clientWidth: number;
    clientHeight: number;
    scrollWidth: number;
    scrollHeight: number;
    offsetWidth: number;
    offsetHeight: number;
  };
  visualViewport: {
    width: number | null;
    height: number | null;
    offsetLeft: number | null;
    offsetTop: number | null;
    scale: number | null;
  } | null;
}

interface DebugOverlayProps {
  isVisible: boolean;
  onToggle: () => void;
}

export const DebugOverlay: React.FC<DebugOverlayProps> = ({ isVisible, onToggle }) => {
  const [measurements, setMeasurements] = useState<Measurements | null>(null);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['screen', 'window']));

  const getMeasurements = (): Measurements => {
    const visualViewport = window.visualViewport ? {
      width: window.visualViewport.width,
      height: window.visualViewport.height,
      offsetLeft: window.visualViewport.offsetLeft,
      offsetTop: window.visualViewport.offsetTop,
      scale: window.visualViewport.scale,
    } : null;

    return {
      screen: {
        width: window.screen.width,
        height: window.screen.height,
        availWidth: window.screen.availWidth,
        availHeight: window.screen.availHeight,
        colorDepth: window.screen.colorDepth,
        pixelDepth: window.screen.pixelDepth,
      },
      window: {
        innerWidth: window.innerWidth,
        innerHeight: window.innerHeight,
        outerWidth: window.outerWidth,
        outerHeight: window.outerHeight,
        devicePixelRatio: window.devicePixelRatio,
        scrollX: window.scrollX,
        scrollY: window.scrollY,
        screenX: window.screenX || 0,
        screenY: window.screenY || 0,
      },
      document: {
        clientWidth: document.documentElement.clientWidth,
        clientHeight: document.documentElement.clientHeight,
        scrollWidth: document.documentElement.scrollWidth,
        scrollHeight: document.documentElement.scrollHeight,
        offsetWidth: document.body.offsetWidth,
        offsetHeight: document.body.offsetHeight,
      },
      visualViewport,
    };
  };

  const updateMeasurements = () => {
    setMeasurements(getMeasurements());
  };

  useEffect(() => {
    if (!isVisible) return;

    updateMeasurements();

    const handleResize = () => updateMeasurements();
    const handleScroll = () => updateMeasurements();
    const handleOrientationChange = () => {
      setTimeout(updateMeasurements, 100); // Delay to allow orientation change to complete
    };

    window.addEventListener('resize', handleResize);
    window.addEventListener('scroll', handleScroll);
    window.addEventListener('orientationchange', handleOrientationChange);

    if (window.visualViewport) {
      window.visualViewport.addEventListener('resize', handleResize);
      window.visualViewport.addEventListener('scroll', handleScroll);
    }

    return () => {
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('scroll', handleScroll);
      window.removeEventListener('orientationchange', handleOrientationChange);

      if (window.visualViewport) {
        window.visualViewport.removeEventListener('resize', handleResize);
        window.visualViewport.removeEventListener('scroll', handleScroll);
      }
    };
  }, [isVisible]);

  const toggleSection = (section: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(section)) {
      newExpanded.delete(section);
    } else {
      newExpanded.add(section);
    }
    setExpandedSections(newExpanded);
  };

  const renderMeasurementRow = (label: string, value: number | null, unit: string = 'px') => (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '2px 0' }}>
      <span style={{ fontSize: '11px', color: '#666' }}>{label}:</span>
      <span style={{ fontSize: '11px', fontWeight: 'bold', color: '#000' }}>
        {value !== null ? `${value}${unit}` : 'N/A'}
      </span>
    </div>
  );

  const renderSection = (title: string, sectionKey: string, content: React.ReactNode, color: string) => {
    const isExpanded = expandedSections.has(sectionKey);
    return (
      <div style={{ marginBottom: '8px', border: `1px solid ${color}`, borderRadius: '4px' }}>
        <div
          style={{
            background: color,
            color: 'white',
            padding: '4px 8px',
            fontSize: '12px',
            fontWeight: 'bold',
            cursor: 'pointer',
            userSelect: 'none',
          }}
          onClick={() => toggleSection(sectionKey)}
        >
          {title} {isExpanded ? '▼' : '▶'}
        </div>
        {isExpanded && (
          <div style={{ padding: '6px 8px', background: 'rgba(255,255,255,0.9)' }}>
            {content}
          </div>
        )}
      </div>
    );
  };

  if (!isVisible || !measurements) return null;

  const overlayStyle: React.CSSProperties = {
    position: 'fixed',
    top: '10px',
    right: '10px',
    width: '280px',
    maxHeight: '90vh',
    backgroundColor: 'rgba(255, 255, 255, 0.95)',
    border: '2px solid #333',
    borderRadius: '8px',
    boxShadow: '0 4px 20px rgba(0,0,0,0.3)',
    zIndex: 10000,
    fontFamily: 'monospace',
    overflowY: 'auto',
  };

  const headerStyle: React.CSSProperties = {
    background: '#333',
    color: 'white',
    padding: '8px 12px',
    fontSize: '14px',
    fontWeight: 'bold',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  };

  const closeButtonStyle: React.CSSProperties = {
    background: 'none',
    border: 'none',
    color: 'white',
    fontSize: '16px',
    cursor: 'pointer',
    padding: '0',
    width: '20px',
    height: '20px',
  };

  return (
    <div style={overlayStyle}>
      <div style={headerStyle}>
        <span>Debug Measurements</span>
        <button style={closeButtonStyle} onClick={onToggle}>×</button>
      </div>
      <div style={{ padding: '8px' }}>
        {renderSection(
          'Screen',
          'screen',
          <>
            {renderMeasurementRow('Width', measurements.screen.width)}
            {renderMeasurementRow('Height', measurements.screen.height)}
            {renderMeasurementRow('Available Width', measurements.screen.availWidth)}
            {renderMeasurementRow('Available Height', measurements.screen.availHeight)}
            {renderMeasurementRow('Color Depth', measurements.screen.colorDepth, 'bit')}
            {renderMeasurementRow('Pixel Depth', measurements.screen.pixelDepth, 'bit')}
          </>,
          '#e74c3c'
        )}

        {renderSection(
          'Window',
          'window',
          <>
            {renderMeasurementRow('Inner Width', measurements.window.innerWidth)}
            {renderMeasurementRow('Inner Height', measurements.window.innerHeight)}
            {renderMeasurementRow('Outer Width', measurements.window.outerWidth)}
            {renderMeasurementRow('Outer Height', measurements.window.outerHeight)}
            {renderMeasurementRow('Device Pixel Ratio', measurements.window.devicePixelRatio, 'x')}
            {renderMeasurementRow('Scroll X', measurements.window.scrollX)}
            {renderMeasurementRow('Scroll Y', measurements.window.scrollY)}
            {renderMeasurementRow('Screen X', measurements.window.screenX)}
            {renderMeasurementRow('Screen Y', measurements.window.screenY)}
          </>,
          '#3498db'
        )}

        {renderSection(
          'Document',
          'document',
          <>
            {renderMeasurementRow('Client Width', measurements.document.clientWidth)}
            {renderMeasurementRow('Client Height', measurements.document.clientHeight)}
            {renderMeasurementRow('Scroll Width', measurements.document.scrollWidth)}
            {renderMeasurementRow('Scroll Height', measurements.document.scrollHeight)}
            {renderMeasurementRow('Body Offset Width', measurements.document.offsetWidth)}
            {renderMeasurementRow('Body Offset Height', measurements.document.offsetHeight)}
          </>,
          '#2ecc71'
        )}

        {measurements.visualViewport && renderSection(
          'Visual Viewport',
          'visualViewport',
          <>
            {renderMeasurementRow('Width', measurements.visualViewport.width)}
            {renderMeasurementRow('Height', measurements.visualViewport.height)}
            {renderMeasurementRow('Offset Left', measurements.visualViewport.offsetLeft)}
            {renderMeasurementRow('Offset Top', measurements.visualViewport.offsetTop)}
            {renderMeasurementRow('Scale', measurements.visualViewport.scale, 'x')}
          </>,
          '#f39c12'
        )}

        <div style={{ 
          marginTop: '8px', 
          padding: '4px', 
          background: '#f8f9fa', 
          borderRadius: '4px',
          fontSize: '10px',
          color: '#666',
          textAlign: 'center'
        }}>
          Press Ctrl+Shift+D to toggle
        </div>
      </div>
    </div>
  );
};