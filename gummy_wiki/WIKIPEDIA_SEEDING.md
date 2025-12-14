# Wikipedia Seeding Guide

This guide explains how to seed your Laravel application with real data from Wikipedia.

## Overview

The Wikipedia seeder fetches real articles from Wikipedia's REST API and populates your database with:
- **Content**: Blog post-like articles (40 articles)
- **Entities**: Structured entities with descriptions (15 entities)
- **Dictionary**: Technical terms with definitions (20 entries)

## Quick Start

### 1. Seed with Wikipedia Data

```bash
php artisan seed:wikipedia
```

This will:
- Fetch ~40 Wikipedia articles for Content model
- Fetch ~15 articles for Entity model
- Fetch ~20 technical terms for Dictionary model
- Display progress as it fetches data

### 2. Create Search Indices

```bash
php artisan elastic:create-index
```

### 3. Import to Search Engine

```bash
php artisan scout:import
```

## Using in DatabaseSeeder

You can also use Wikipedia data in your regular seeder:

**Option 1: Environment Variable**

Add to `.env`:
```env
USE_WIKIPEDIA_SEEDER=true
```

Then run:
```bash
php artisan db:seed
```

**Option 2: Direct Call**

Modify `database/seeders/DatabaseSeeder.php`:
```php
public function run(): void
{
    $this->call(WikipediaSeeder::class);

    // Still create files with factory
    File::factory(10)->create();
}
```

## Command Options

```bash
# Seed only Content model
php artisan seed:wikipedia --content-only

# Seed only Entity model
php artisan seed:wikipedia --entities-only

# Seed only Dictionary model
php artisan seed:wikipedia --dictionary-only
```

## Articles Included

### Content Articles (40)
- Technology: Rust, Laravel, Elasticsearch, Search engine, Database, API, JSON, HTTP, REST, Microservices
- Science: Quantum computing, AI, Machine learning, Neural network, Algorithm, Data structure, Computer science, Software engineering, Open source, Version control
- History & Geography: World War II, Renaissance, Industrial Revolution, United States, Europe, Asia, Africa, Pacific Ocean, Mount Everest, Amazon River
- Arts & Culture: Leonardo da Vinci, Shakespeare, Mozart, Picasso, Literature, Music, Painting, Sculpture, Architecture, Philosophy

### Dictionary Terms (20)
Technical terms: Algorithm, Database, API, JSON, HTTP, REST, Cache, Index, Query, Search, Token, Parser, Compiler, Interpreter, Framework, Library, Repository, Deployment, Scalability, Performance

## Rate Limiting

The seeder includes a 200ms delay between API requests to be respectful to Wikipedia's servers. This means:
- ~40 Content articles: ~8 seconds
- ~15 Entity articles: ~3 seconds
- ~20 Dictionary terms: ~4 seconds
- **Total time: ~15-20 seconds**

## Error Handling

If an article fails to fetch (e.g., doesn't exist or API error), the seeder will:
- Display a warning message
- Continue with the next article
- Report the final count of successfully created records

## Customization

### Add More Articles

Edit `database/seeders/WikipediaSeeder.php`:

```php
private array $articleTitles = [
    'Your Article Title',
    'Another Article',
    // ... add more
];
```

### Change Article Count

Modify the arrays in `WikipediaSeeder.php`:
- `$articleTitles` - for Content and Entity models
- `$dictionaryTerms` - for Dictionary model

### Adjust Rate Limiting

Change the delay in the seeder methods:
```php
usleep(200000); // 200ms - increase for slower requests
```

## Testing Search

After seeding, test your search:

```bash
# Search via command
php artisan search:content "Rust"

# Or in tinker
php artisan tinker
>>> Content::search('programming')->get();
>>> Entity::search('technology')->get();
>>> Dictionary::search('API')->get();
```

## Notes

- Wikipedia API is free and doesn't require authentication
- Articles are fetched in English (en.wikipedia.org)
- Only "standard" articles are used (disambiguation pages are skipped)
- The seeder uses Wikipedia's summary API which provides concise extracts
- All data is real and up-to-date from Wikipedia

## Troubleshooting

### "Connection timeout" errors
- Check your internet connection
- Wikipedia API might be temporarily unavailable
- Try running the seeder again

### "No articles created"
- Verify Wikipedia API is accessible: `curl https://en.wikipedia.org/api/rest_v1/page/summary/Rust`
- Check that article titles exist on Wikipedia
- Review error messages in the output

### Rate limiting issues
- Increase the `usleep()` delay if you get rate-limited
- Wikipedia allows reasonable use, but very fast requests might be throttled

## Next Steps

1. **Test Search Functionality**: Try searching for various terms
2. **Add More Data**: Customize the article lists for your needs
3. **Performance Testing**: Use the seeded data to test search performance
4. **Integration Testing**: Use real data for more realistic test scenarios
