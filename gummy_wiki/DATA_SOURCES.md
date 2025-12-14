# Free Text Databases for Testing

This document lists free text databases and datasets that can be used to seed Gummy Wiki with realistic data for testing Gummy Search.

## Content (Blog Posts / Articles)

### 1. **Project Gutenberg** (Classic Literature)
- **URL**: https://www.gutenberg.org/
- **Format**: Plain text, HTML, EPUB
- **Content**: 70,000+ free ebooks (public domain)
- **Use Case**: Extract chapters/sections as blog posts
- **License**: Public domain
- **API**: No official API, but can download via FTP or web scraping

**Example Usage:**
```bash
# Download a book
wget https://www.gutenberg.org/files/1342/1342-0.txt  # Pride and Prejudice

# Split into chunks and import
```

### 2. **Wikipedia Dumps**
- **URL**: https://dumps.wikimedia.org/
- **Format**: XML, JSON
- **Content**: Full Wikipedia articles
- **Use Case**: Real-world articles with rich content
- **License**: CC BY-SA 3.0
- **Size**: Large (multi-GB), but can extract specific articles

**Example Usage:**
```bash
# Download a small sample
wget https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-pages-articles1.xml-p1p30303.bz2

# Parse and extract articles
```

### 3. **Common Crawl** (Web Content)
- **URL**: https://commoncrawl.org/
- **Format**: WARC files
- **Content**: Billions of web pages
- **Use Case**: Diverse real-world content
- **License**: Varies by source
- **Size**: Very large (terabytes)

### 4. **News Articles Datasets**
- **Kaggle News Datasets**: https://www.kaggle.com/datasets?search=news
- **Format**: CSV, JSON
- **Content**: News articles from various sources
- **Examples**:
  - "News Category Dataset" (200k+ news articles)
  - "BBC News Dataset"
  - "Reuters News Dataset"

### 5. **Reddit Datasets**
- **URL**: https://www.reddit.com/r/datasets/
- **Format**: JSON
- **Content**: Reddit posts and comments
- **Use Case**: Modern conversational content
- **License**: Reddit API Terms

## Dictionary Entries

### 1. **Wiktionary Dumps**
- **URL**: https://dumps.wikimedia.org/wiktionary/
- **Format**: XML, JSON
- **Content**: Dictionary definitions from Wiktionary
- **Use Case**: Perfect for Dictionary model
- **License**: CC BY-SA 3.0

### 2. **Free Dictionary API**
- **URL**: https://dictionaryapi.dev/
- **Format**: JSON API
- **Content**: Word definitions
- **Use Case**: Real-time dictionary lookups
- **License**: Free for non-commercial use

### 3. **WordNet**
- **URL**: https://wordnet.princeton.edu/
- **Format**: Database files
- **Content**: Lexical database of English
- **Use Case**: Synonyms, definitions, relationships
- **License**: Open source

### 4. **GCIDE (GNU Collaborative International Dictionary of English)**
- **URL**: https://gcide.gnu.org.ua/
- **Format**: XML
- **Content**: Comprehensive dictionary
- **License**: GPL

## Entity Descriptions

### 1. **DBpedia** (Structured Wikipedia)
- **URL**: https://www.dbpedia.org/
- **Format**: RDF, JSON-LD, CSV
- **Content**: Structured data extracted from Wikipedia
- **Use Case**: Entities with descriptions, properties
- **License**: CC BY-SA 3.0
- **API**: SPARQL endpoint available

**Example SPARQL Query:**
```sparql
SELECT ?entity ?label ?description WHERE {
  ?entity rdfs:label ?label .
  ?entity rdfs:comment ?description .
  FILTER (LANG(?label) = "en")
  FILTER (LANG(?description) = "en")
} LIMIT 1000
```

### 2. **Wikidata**
- **URL**: https://www.wikidata.org/
- **Format**: JSON, RDF
- **Content**: Structured data about entities
- **Use Case**: Rich entity data with properties
- **License**: CC0 (public domain)
- **API**: REST API and SPARQL endpoint

### 3. **GeoNames**
- **URL**: https://www.geonames.org/
- **Format**: CSV, XML, JSON
- **Content**: Geographic entities (cities, countries, etc.)
- **Use Case**: Location-based entities
- **License**: Creative Commons Attribution 4.0

## File Metadata

### 1. **Open Images Dataset**
- **URL**: https://storage.googleapis.com/openimages/web/index.html
- **Format**: CSV, JSON
- **Content**: Image metadata (filenames, descriptions, labels)
- **Use Case**: File model with image metadata
- **License**: CC BY 4.0

### 2. **Common Crawl File Lists**
- **URL**: https://commoncrawl.org/
- **Format**: CSV
- **Content**: File metadata from web crawls
- **Use Case**: Diverse file types and metadata

## Quick Start: Using Sample Datasets

### Option 1: Use Pre-processed JSON/CSV Files

Create a seeder that reads from JSON/CSV files:

```php
// database/seeders/RealDataSeeder.php
public function run(): void
{
    // Load from JSON file
    $articles = json_decode(file_get_contents('data/articles.json'), true);

    foreach ($articles as $article) {
        Content::create([
            'title' => $article['title'],
            'body' => $article['content'],
            'published_at' => now(),
        ]);
    }
}
```

### Option 2: Use APIs

```php
// Fetch from API and seed
$response = Http::get('https://api.example.com/articles');
$articles = $response->json();

foreach ($articles as $article) {
    Content::create($article);
}
```

## Recommended Datasets for Quick Testing

### Small/Medium Datasets (Good for Testing)

1. **News Category Dataset** (Kaggle)
   - ~200k news articles
   - Categories: business, tech, sports, etc.
   - Format: CSV
   - Size: ~50MB

2. **Wikipedia Small Sample**
   - Extract 1000 random articles
   - Use Wikipedia API or dumps
   - Format: JSON

3. **Wiktionary Sample**
   - Extract 10k dictionary entries
   - Format: JSON
   - Perfect for Dictionary model

### Large Datasets (For Performance Testing)

1. **Common Crawl Sample**
   - 1M+ web pages
   - Very diverse content
   - Format: WARC

2. **Full Wikipedia Dump**
   - 6M+ articles
   - Comprehensive coverage
   - Format: XML

## Tools for Processing Data

### 1. **jq** (JSON processor)
```bash
# Extract articles from JSON
cat articles.json | jq '.[] | {title: .title, body: .content}'
```

### 2. **Python Scripts**
```python
import json
import requests

# Download and process Wikipedia articles
# Convert to Laravel seeder format
```

### 3. **Laravel Commands**
Create artisan commands to fetch and seed data:
```bash
php artisan seed:from-wikipedia
php artisan seed:from-kaggle
```

## Legal Considerations

- Always check licenses before using datasets
- Some datasets require attribution
- Commercial use may have restrictions
- Public domain content is safest (Project Gutenberg, etc.)

## Example: Wikipedia Article Seeder

Here's a simple example of how to seed from Wikipedia:

```php
// app/Console/Commands/SeedWikipedia.php
use Illuminate\Support\Facades\Http;

public function handle()
{
    $titles = ['Rust', 'Laravel', 'Elasticsearch', 'Search Engine'];

    foreach ($titles as $title) {
        $response = Http::get('https://en.wikipedia.org/api/rest_v1/page/summary/' . urlencode($title));
        $data = $response->json();

        Content::create([
            'title' => $data['title'],
            'body' => $data['extract'],
            'published_at' => now(),
        ]);
    }
}
```

## Resources

- **Kaggle Datasets**: https://www.kaggle.com/datasets
- **Google Dataset Search**: https://datasetsearch.research.google.com/
- **Data.gov**: https://data.gov/ (US government data)
- **EU Open Data Portal**: https://data.europa.eu/
- **Awesome Public Datasets**: https://github.com/awesomedata/awesome-public-datasets
