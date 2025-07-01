import { useState, useEffect } from 'react';

export function useMobileDetection() {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const checkMobile = () => {
      // Check user agent for mobile devices
      const userAgent = navigator.userAgent.toLowerCase();
      const mobileKeywords = [
        'android', 'webos', 'iphone', 'ipad', 'ipod', 'blackberry', 'windows phone'
      ];
      
      const isMobileUserAgent = mobileKeywords.some(keyword => userAgent.includes(keyword));
      
      // Check screen width as secondary indicator
      const isSmallScreen = window.innerWidth <= 768;
      
      // Check for touch capability
      const hasTouchSupport = 'ontouchstart' in window || navigator.maxTouchPoints > 0;
      
      // Consider mobile if user agent indicates mobile OR (small screen AND touch support)
      const mobile = isMobileUserAgent || (isSmallScreen && hasTouchSupport);
      
      setIsMobile(mobile);
    };

    // Check on mount
    checkMobile();

    // Check on resize
    window.addEventListener('resize', checkMobile);
    
    return () => {
      window.removeEventListener('resize', checkMobile);
    };
  }, []);

  return isMobile;
}