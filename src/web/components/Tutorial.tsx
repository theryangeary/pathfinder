import { useState } from 'react';

interface TutorialProps {
  isOpen: boolean;
  onClose: () => void;
  onSkip: () => void;
}

const Tutorial = ({ isOpen, onClose, onSkip }: TutorialProps) => {
  const [currentStep, setCurrentStep] = useState(0);

  const steps = [
    {
      title: "Welcome to Pathfinder!",
      content: (
        <div>
          <p>Find words by connecting adjacent letters on the 4×4 grid.</p>
          <p><strong>Goal:</strong> Score as high as possible by finding 5 words!</p>
        </div>
      )
    },
    {
      title: "How to Connect Letters",
      content: (
        <div>
          <p>Letters can connect <strong>orthogonally</strong> (up, down, left, right) or <strong>diagonally</strong>.</p>
          <p>Trace a path through adjacent tiles to spell words.</p>
        </div>
      )
    },
    {
      title: "Wildcard Tiles",
      content: (
        <div>
          <p>Look for <strong>wildcard tiles</strong> (marked with *) - they can represent any letter!</p>
          <p><strong>Important:</strong> Wildcards must have the same letter value across all your words.</p>
          <p>Wildcards show possible values as they are narrowed down. When paths without wildcards are possible, the wildcard will not be used.</p>
        </div>
      )
    },
    {
      title: "Scoring & Strategy",
      content: (
        <div>
          <p>Letters are worth points based on how rare they are.</p>
          <p>Common letters (like E, A) are worth fewer points than rare letters (like Q, Z).</p>
          <p>Wildcard tiles are worth 0 points, but unlock valuable word possibilities!</p>
        </div>
      )
    }
  ];

  if (!isOpen) return null;

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      onClose();
    }
  };

  const handlePrev = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.7)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
      padding: '20px'
    }}>
      <div style={{
        backgroundColor: 'white',
        borderRadius: '12px',
        padding: '32px',
        maxWidth: '500px',
        width: '100%',
        maxHeight: '80vh',
        overflow: 'auto',
        boxShadow: '0 10px 25px rgba(0, 0, 0, 0.3)'
      }}>
        {/* Header */}
        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '24px'
        }}>
          <h2 style={{
            margin: 0,
            fontSize: '24px',
            color: '#333'
          }}>
            {steps[currentStep].title}
          </h2>
          <button
            onClick={onSkip}
            style={{
              background: 'none',
              border: '1px solid #ddd',
              borderRadius: '6px',
              padding: '6px 12px',
              fontSize: '14px',
              color: '#666',
              cursor: 'pointer'
            }}
          >
            Skip Tutorial
          </button>
        </div>

        {/* Progress indicator */}
        <div style={{
          display: 'flex',
          gap: '8px',
          marginBottom: '24px'
        }}>
          {steps.map((_, index) => (
            <div
              key={index}
              style={{
                flex: 1,
                height: '4px',
                backgroundColor: index <= currentStep ? '#4CAF50' : '#e0e0e0',
                borderRadius: '2px',
                transition: 'background-color 0.3s ease'
              }}
            />
          ))}
        </div>

        {/* Content */}
        <div style={{
          fontSize: '16px',
          lineHeight: '1.6',
          color: '#555',
          marginBottom: '32px',
          minHeight: '120px'
        }}>
          {steps[currentStep].content}
        </div>

        {/* Navigation */}
        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center'
        }}>
          <button
            onClick={handlePrev}
            disabled={currentStep === 0}
            style={{
              backgroundColor: currentStep === 0 ? '#f5f5f5' : '#e0e0e0',
              color: currentStep === 0 ? '#ccc' : '#333',
              border: 'none',
              borderRadius: '8px',
              padding: '12px 24px',
              fontSize: '16px',
              cursor: currentStep === 0 ? 'not-allowed' : 'pointer',
              transition: 'all 0.2s ease'
            }}
          >
            Previous
          </button>

          <span style={{
            fontSize: '14px',
            color: '#666'
          }}>
            {currentStep + 1} of {steps.length}
          </span>

          <button
            onClick={handleNext}
            style={{
              backgroundColor: '#4CAF50',
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              padding: '12px 24px',
              fontSize: '16px',
              cursor: 'pointer',
              transition: 'all 0.2s ease'
            }}
            onMouseOver={(e) => {
              e.currentTarget.style.backgroundColor = '#45a049';
            }}
            onMouseOut={(e) => {
              e.currentTarget.style.backgroundColor = '#4CAF50';
            }}
          >
            {currentStep === steps.length - 1 ? 'Start Playing!' : 'Next'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default Tutorial;
