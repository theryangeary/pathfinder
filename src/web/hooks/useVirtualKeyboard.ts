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

    // Check if Visual Viewport API is available (more reliable for iOS)
    const hasVisualViewport = 'visualViewport' in window;
    
    let initialHeight = 0;
    let initialViewportHeight = 0;
    
    const setInitialHeights = () => {
      initialHeight = window.innerHeight;
      if (hasVisualViewport) {
        initialViewportHeight = window.visualViewport!.height;
      }
    };
    
    // Set initial heights after a short delay to account for browser UI settling
    setTimeout(setInitialHeights, 100);

    const checkKeyboardVisibility = () => {
      // If we don't have initial heights yet, set them now
      if (initialHeight === 0) {
        setInitialHeights();
        return;
      }

      let isKeyboardVisible = false;

      if (hasVisualViewport) {
        // Use Visual Viewport API for more reliable detection (especially on iOS)
        const currentViewportHeight = window.visualViewport!.height;
        const viewportHeightDifference = initialViewportHeight - currentViewportHeight;
        
        // Also check window height as fallback
        const currentHeight = window.innerHeight;
        const windowHeightDifference = initialHeight - currentHeight;
        
        // Keyboard is visible if either viewport height decreased significantly
        // or window height decreased significantly
        isKeyboardVisible = viewportHeightDifference > 150 || windowHeightDifference > 150;
      } else {
        // Fallback to window height for browsers without Visual Viewport API
        const currentHeight = window.innerHeight;
        const heightDifference = initialHeight - currentHeight;
        isKeyboardVisible = heightDifference > 150;
      }
      
      setKeyboardState({
        isVisible: isKeyboardVisible,
      });
    };

    // Initial check
    checkKeyboardVisibility();

    // Listen for window resize events
    window.addEventListener('resize', checkKeyboardVisibility);
    
    // Listen for Visual Viewport changes if available
    if (hasVisualViewport) {
      window.visualViewport!.addEventListener('resize', checkKeyboardVisibility);
    }
    
    // Also listen for orientation changes which might reset our baseline
    const handleOrientationChange = () => {
      // Reset initial heights after orientation change
      setTimeout(() => {
        setInitialHeights();
        checkKeyboardVisibility();
      }, 500); // Wait for orientation change to complete
    };
    
    window.addEventListener('orientationchange', handleOrientationChange);

    return () => {
      window.removeEventListener('resize', checkKeyboardVisibility);
      if (hasVisualViewport) {
        window.visualViewport!.removeEventListener('resize', checkKeyboardVisibility);
      }
      window.removeEventListener('orientationchange', handleOrientationChange);
    };
  }, []);

  return keyboardState;
};
