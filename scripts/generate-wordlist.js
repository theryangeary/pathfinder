#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const WORDLIST_SOURCE = path.join(__dirname, '../src/api/wordlist');
const FRONTEND_OUTPUT = path.join(__dirname, '../src/web/data/wordList.ts');

function generateWordlist() {
  try {
    console.log('📚 Generating wordlist from:', WORDLIST_SOURCE);
    
    // Read the wordlist file
    const wordlistContent = fs.readFileSync(WORDLIST_SOURCE, 'utf8');
    const words = wordlistContent
      .split('\n')
      .map(word => word.trim().toLowerCase())
      .filter(word => word.length > 0);
    
    console.log(`   Found ${words.length} words`);
    
    // Generate TypeScript module
    const tsContent = `// This file is auto-generated from src/api/wordlist
// Do not edit manually - run 'npm run generate-wordlist' to regenerate

const validWords = new Set([
${words.map(word => `  "${word}"`).join(',\n')}
]);

export function isValidWord(word: string): boolean {
  return validWords.has(word.toLowerCase());
}

export { validWords };
`;

    // Ensure output directory exists
    const outputDir = path.dirname(FRONTEND_OUTPUT);
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }
    
    // Write the generated file
    fs.writeFileSync(FRONTEND_OUTPUT, tsContent);
    
    console.log('✅ Generated frontend wordlist:', FRONTEND_OUTPUT);
    console.log(`   Exported ${words.length} words for frontend validation`);
    
  } catch (error) {
    console.error('❌ Error generating wordlist:', error.message);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  generateWordlist();
}

module.exports = { generateWordlist };