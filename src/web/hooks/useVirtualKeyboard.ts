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
    if (typeof window === 'undefined') return;

    // Store initial window height to detect changes
    // Use a small delay to get accurate initial height after page load
    let initialHeight = 0;
    const setInitialHeight = () => {
      initialHeight = window.innerHeight;
    };
    
    // Set initial height after a short delay to account for browser UI settling
    setTimeout(setInitialHeight, 100);

    const handleResize = () => {
      // If we don't have initial height yet, set it now
      if (initialHeight === 0) {
        initialHeight = window.innerHeight;
        return;
      }

      const currentHeight = window.innerHeight;
      const heightDifference = initialHeight - currentHeight;
      
      // With interactive-widget=resizes-content, the window height shrinks when keyboard appears
      // Consider keyboard visible if window height is significantly smaller than initial
      // Use 150px threshold to account for browser UI changes and small screen rotations
      const isKeyboardVisible = heightDifference > 150;
      
      setKeyboardState({
        isVisible: isKeyboardVisible,
        height: isKeyboardVisible ? heightDifference : 0
      });
    };

    // Initial check
    handleResize();

    // Listen for window resize events
    window.addEventListener('resize', handleResize);
    
    // Also listen for orientation changes which might reset our baseline
    const handleOrientationChange = () => {
      // Reset initial height after orientation change
      setTimeout(() => {
        initialHeight = window.innerHeight;
        handleResize();
      }, 500); // Wait for orientation change to complete
    };
    
    window.addEventListener('orientationchange', handleOrientationChange);

    return () => {
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('orientationchange', handleOrientationChange);
    };
  }, []);

  return keyboardState;
};
