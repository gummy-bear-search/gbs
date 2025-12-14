<?php

namespace Database\Seeders;

use App\Models\Content;
use App\Models\Dictionary;
use App\Models\Entity;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Str;

class WikipediaSeeder extends Seeder
{
    /**
     * Command instance for output (set when called from command)
     */
    public $command;

    /**
     * List of Wikipedia article titles to fetch
     * Mix of technology, science, history, geography, etc.
     */
    private array $articleTitles = [
        // Technology
        'Rust (programming language)',
        'Laravel',
        'Elasticsearch',
        'Search engine',
        'Database',
        'API',
        'JSON',
        'HTTP',
        'REST',
        'Microservices',

        // Science
        'Quantum computing',
        'Artificial intelligence',
        'Machine learning',
        'Neural network',
        'Algorithm',
        'Data structure',
        'Computer science',
        'Software engineering',
        'Open source',
        'Version control',

        // History & Geography
        'World War II',
        'Renaissance',
        'Industrial Revolution',
        'United States',
        'Europe',
        'Asia',
        'Africa',
        'Pacific Ocean',
        'Mount Everest',
        'Amazon River',

        // Arts & Culture
        'Leonardo da Vinci',
        'Shakespeare',
        'Mozart',
        'Picasso',
        'Literature',
        'Music',
        'Painting',
        'Sculpture',
        'Architecture',
        'Philosophy',
    ];

    /**
     * Dictionary terms to fetch from Wikipedia
     */
    private array $dictionaryTerms = [
        'Algorithm',
        'Database',
        'API',
        'JSON',
        'HTTP',
        'REST',
        'Cache',
        'Index',
        'Query',
        'Search',
        'Token',
        'Parser',
        'Compiler',
        'Interpreter',
        'Framework',
        'Library',
        'Repository',
        'Deployment',
        'Scalability',
        'Performance',
    ];

    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        $output = $this->command ?? $this;

        $output->info('Fetching Wikipedia articles...');

        // Seed Content from Wikipedia articles
        $this->seedContent($output);

        // Seed Entities from Wikipedia articles
        $this->seedEntities($output);

        // Seed Dictionary from Wikipedia/Wiktionary
        $this->seedDictionary($output);

        $output->info('Wikipedia seeding completed!');
    }

    /**
     * Seed Content model with Wikipedia articles
     */
    private function seedContent($output = null): void
    {
        $output = $output ?? $this->command ?? $this;

        $output->info('Seeding Content from Wikipedia...');

        $count = 0;
        foreach ($this->articleTitles as $title) {
            try {
                $article = $this->fetchWikipediaArticle($title);

                if ($article) {
                    Content::create([
                        'title' => $article['title'],
                        'body' => $article['extract'],
                        'published_at' => now()->subDays(rand(1, 365)),
                    ]);

                    $count++;
                    $output->line("  ✓ Fetched: {$article['title']}");

                    // Rate limiting - be nice to Wikipedia API
                    usleep(200000); // 200ms delay between requests
                }
            } catch (\Exception $e) {
                $output->warn("  ✗ Failed to fetch '{$title}': {$e->getMessage()}");
            }
        }

        $output->info("Created {$count} content articles from Wikipedia.");
    }

    /**
     * Seed Entity model with Wikipedia articles
     */
    private function seedEntities($output = null): void
    {
        $output = $output ?? $this->command ?? $this;

        $output->info('Seeding Entities from Wikipedia...');

        // Use a subset of articles for entities
        $entityTitles = array_slice($this->articleTitles, 0, 15);
        $entityTypes = ['person', 'place', 'concept', 'technology', 'event', 'organization'];

        $count = 0;
        foreach ($entityTitles as $title) {
            try {
                $article = $this->fetchWikipediaArticle($title);

                if ($article) {
                    // Determine entity type based on title or random
                    $type = $this->determineEntityType($article['title'], $entityTypes);

                    Entity::create([
                        'name' => $article['title'],
                        'description' => $article['extract'],
                        'type' => $type,
                    ]);

                    $count++;
                    $output->line("  ✓ Created entity: {$article['title']} ({$type})");

                    usleep(200000); // Rate limiting
                }
            } catch (\Exception $e) {
                $output->warn("  ✗ Failed to fetch entity '{$title}': {$e->getMessage()}");
            }
        }

        $output->info("Created {$count} entities from Wikipedia.");
    }

    /**
     * Seed Dictionary model with Wikipedia/Wiktionary definitions
     */
    private function seedDictionary($output = null): void
    {
        $output = $output ?? $this->command ?? $this;

        $output->info('Seeding Dictionary from Wikipedia...');

        $count = 0;
        foreach ($this->dictionaryTerms as $term) {
            try {
                $article = $this->fetchWikipediaArticle($term);

                if ($article) {
                    // Use first paragraph or extract as definition
                    $definition = $this->extractDefinition($article['extract']);

                    Dictionary::create([
                        'term' => $article['title'],
                        'definition' => $definition,
                        'language' => 'en',
                    ]);

                    $count++;
                    $output->line("  ✓ Added dictionary entry: {$article['title']}");

                    usleep(200000); // Rate limiting
                }
            } catch (\Exception $e) {
                $output->warn("  ✗ Failed to fetch dictionary term '{$term}': {$e->getMessage()}");
            }
        }

        $output->info("Created {$count} dictionary entries from Wikipedia.");
    }

    /**
     * Fetch article from Wikipedia API
     */
    private function fetchWikipediaArticle(string $title): ?array
    {
        $url = 'https://en.wikipedia.org/api/rest_v1/page/summary/' . urlencode($title);

        try {
            $response = Http::timeout(10)->get($url);

            if ($response->successful()) {
                $data = $response->json();

                // Check if page exists (not a disambiguation or missing page)
                if (isset($data['type']) && $data['type'] === 'standard' && isset($data['extract'])) {
                    return [
                        'title' => $data['title'] ?? $title,
                        'extract' => $data['extract'] ?? '',
                        'url' => $data['content_urls']['desktop']['page'] ?? '',
                    ];
                }
            }
        } catch (\Exception $e) {
            throw new \Exception("API request failed: " . $e->getMessage());
        }

        return null;
    }

    /**
     * Extract a concise definition from Wikipedia extract
     */
    private function extractDefinition(string $extract): string
    {
        // Take first sentence or first 500 characters
        $firstSentence = Str::before($extract, '.');

        if (strlen($firstSentence) > 500) {
            return Str::limit($extract, 500);
        }

        return $firstSentence ?: Str::limit($extract, 500);
    }

    /**
     * Determine entity type based on title
     */
    private function determineEntityType(string $title, array $types): string
    {
        $titleLower = strtolower($title);

        // Simple heuristics
        if (Str::contains($titleLower, ['programming', 'language', 'software', 'framework', 'api'])) {
            return 'technology';
        }

        if (Str::contains($titleLower, ['war', 'revolution', 'battle'])) {
            return 'event';
        }

        if (Str::contains($titleLower, ['ocean', 'river', 'mountain', 'country', 'continent'])) {
            return 'place';
        }

        // Check if it's a person name (has common name patterns)
        $words = explode(' ', $title);
        if (count($words) >= 2 && strlen($words[0]) < 15) {
            return 'person';
        }

        // Default to random type
        return $types[array_rand($types)];
    }
}
