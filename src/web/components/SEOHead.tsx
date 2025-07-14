import { useEffect } from 'react';
import { useLocation, useParams } from 'react-router-dom';

interface SEOHeadProps {
  gameDate?: string;
  isCompleted?: boolean;
  totalScore?: number;
}

const SEOHead: React.FC<SEOHeadProps> = ({ 
  gameDate, 
  isCompleted, 
  totalScore 
}) => {
  const location = useLocation();
  const { sequenceNumber: urlSequenceNumber } = useParams();
  
  useEffect(() => {
    const baseUrl = 'https://pathfinder.prof';
    const currentUrl = `${baseUrl}${location.pathname}`;
    
    // Update canonical URL
    let canonicalLink = document.querySelector('link[rel="canonical"]') as HTMLLinkElement;
    if (!canonicalLink) {
      canonicalLink = document.createElement('link');
      canonicalLink.rel = 'canonical';
      document.head.appendChild(canonicalLink);
    }
    canonicalLink.href = currentUrl;
    
    // Update page title based on context
    let title = 'Pathfinder - Daily Word Puzzle Game | Free Online Word Game';
    let description = 'Play Pathfinder, the daily word puzzle game! Find words on a 4x4 grid by connecting adjacent letters. New puzzle every day. Free to play!';
    
    if (urlSequenceNumber) {
      const puzzleNumber = urlSequenceNumber;
      title = `Puzzle #${puzzleNumber} - Pathfinder Daily Word Game`;
      description = `Play Pathfinder puzzle #${puzzleNumber}. Find words on a 4x4 grid by connecting adjacent letters in this daily word challenge.`;
      
      if (gameDate) {
        title = `Puzzle #${puzzleNumber} (${gameDate}) - Pathfinder`;
        description = `Play Pathfinder puzzle #${puzzleNumber} from ${gameDate}. Find words on a 4x4 grid by connecting adjacent letters.`;
      }
      
      if (isCompleted && totalScore !== undefined) {
        title = `Puzzle #${puzzleNumber} Complete! Score: ${totalScore} - Pathfinder`;
        description = `Completed Pathfinder puzzle #${puzzleNumber} with a score of ${totalScore} points. Try today's puzzle!`;
      }
    }
    
    // Update title
    document.title = title;
    
    // Update meta description
    let metaDescription = document.querySelector('meta[name="description"]') as HTMLMetaElement;
    if (metaDescription) {
      metaDescription.content = description;
    }
    
    // Update Open Graph tags
    let ogTitle = document.querySelector('meta[property="og:title"]') as HTMLMetaElement;
    if (ogTitle) {
      ogTitle.content = title;
    }
    
    let ogDescription = document.querySelector('meta[property="og:description"]') as HTMLMetaElement;
    if (ogDescription) {
      ogDescription.content = description;
    }
    
    let ogUrl = document.querySelector('meta[property="og:url"]') as HTMLMetaElement;
    if (ogUrl) {
      ogUrl.content = currentUrl;
    }
    
    // Update Twitter tags
    let twitterTitle = document.querySelector('meta[property="twitter:title"]') as HTMLMetaElement;
    if (twitterTitle) {
      twitterTitle.content = title;
    }
    
    let twitterDescription = document.querySelector('meta[property="twitter:description"]') as HTMLMetaElement;
    if (twitterDescription) {
      twitterDescription.content = description;
    }
    
    let twitterUrl = document.querySelector('meta[property="twitter:url"]') as HTMLMetaElement;
    if (twitterUrl) {
      twitterUrl.content = currentUrl;
    }
    
    // Update structured data
    const structuredDataScript = document.querySelector('script[type="application/ld+json"]');
    if (structuredDataScript && urlSequenceNumber) {
      try {
        const structuredData = JSON.parse(structuredDataScript.textContent || '{}');
        
        // Add specific puzzle information
        structuredData.hasPart = [{
          "@type": "Game",
          "name": `Daily Pathfinder Puzzle #${urlSequenceNumber}`,
          "description": description,
          "gameLocation": "Online",
          "numberOfPlayers": "1",
          "url": currentUrl
        }];
        
        if (gameDate) {
          structuredData.hasPart[0].datePublished = gameDate;
        }
        
        structuredDataScript.textContent = JSON.stringify(structuredData);
      } catch (error) {
        console.warn('Failed to update structured data:', error);
      }
    }
    
  }, [location.pathname, urlSequenceNumber, gameDate, isCompleted, totalScore]);
  
  return null; // This component doesn't render anything
};

export default SEOHead;
