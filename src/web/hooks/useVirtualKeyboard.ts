import { useEffect, useState } from 'react';

interface VirtualKeyboardState {
  isVisible: boolean;
  height: number;
}

export const useVirtualKeyboard = (): VirtualKeyboardState => {
  const [keyboardState, setKeyboardState] = useState<VirtualKeyboardState>({
    isVisible: false,
    height: 0
  });

  useEffect(() => {
    // Check if the Visual Viewport API is supported
    if (typeof window !== 'undefined' && 'visualViewport' in window) {
      const visualViewport = window.visualViewport!;
      
      const handleViewportChange = () => {
        const windowHeight = window.innerHeight;
        const viewportHeight = visualViewport.height;
        const heightDifference = windowHeight - viewportHeight;
        
        // Consider keyboard visible if the viewport height is significantly smaller
        // than the window height (threshold of 150px to account for browser UI)
        const isKeyboardVisible = heightDifference > 150;
        
        setKeyboardState({
          isVisible: isKeyboardVisible,
          height: isKeyboardVisible ? heightDifference : 0
        });
      };

      // Initial check
      handleViewportChange();

      // Listen for viewport changes
      visualViewport.addEventListener('resize', handleViewportChange);
      visualViewport.addEventListener('scroll', handleViewportChange);

      return () => {
        visualViewport.removeEventListener('resize', handleViewportChange);
        visualViewport.removeEventListener('scroll', handleViewportChange);
      };
    } else {
      // Fallback for browsers without Visual Viewport API
      const handleResize = () => {
        const currentHeight = window.innerHeight;
        const screenHeight = window.screen.height;
        
        // Detect keyboard by significant height reduction
        // This is less reliable but works as a fallback
        const heightReduction = screenHeight - currentHeight;
        const isKeyboardVisible = heightReduction > 300; // More conservative threshold
        
        setKeyboardState({
          isVisible: isKeyboardVisible,
          height: isKeyboardVisible ? heightReduction : 0
        });
      };

      // Initial check
      handleResize();

      window.addEventListener('resize', handleResize);
      
      return () => {
        window.removeEventListener('resize', handleResize);
      };
    }
  }, []);

  return keyboardState;
};