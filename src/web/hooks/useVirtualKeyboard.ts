import { useEffect, useState } from 'react';

interface VirtualKeyboardState {
  isVisible: boolean;
}

export const useVirtualKeyboard = (): VirtualKeyboardState => {
  const [keyboardState, setKeyboardState] = useState<VirtualKeyboardState>({
    isVisible: false,
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    // Check if Visual Viewport API is available
    const hasVisualViewport = 'visualViewport' in window;
    
    // Keyboard detection cutoff: 65.5% of screen height
    // Based on measured ranges: no-keyboard ~82%, keyboard ~49%
    const KEYBOARD_CUTOFF_RATIO = 0.655;
    
    const checkKeyboardVisibility = () => {
      let isKeyboardVisible = false;

      if (hasVisualViewport && 'screen' in window) {
        const screenHeight = window.screen.height;
        const viewportHeight = window.visualViewport!.height;
        const viewportRatio = viewportHeight / screenHeight;
        
        isKeyboardVisible = viewportRatio < KEYBOARD_CUTOFF_RATIO;
      } else {
        // Fallback for browsers without Visual Viewport API or screen info
        // Use a simple height threshold approach
        const currentHeight = window.innerHeight;
        isKeyboardVisible = currentHeight < 500; // Conservative fallback
      }
      
      setKeyboardState({
        isVisible: isKeyboardVisible,
      });
    };

    // Initial check
    checkKeyboardVisibility();

    // Listen for Visual Viewport changes if available
    if (hasVisualViewport) {
      window.visualViewport!.addEventListener('resize', checkKeyboardVisibility);
    }
    
    // Listen for window resize as fallback
    window.addEventListener('resize', checkKeyboardVisibility);

    return () => {
      if (hasVisualViewport) {
        window.visualViewport!.removeEventListener('resize', checkKeyboardVisibility);
      }
      window.removeEventListener('resize', checkKeyboardVisibility);
    };
  }, []);

  return keyboardState;
};
