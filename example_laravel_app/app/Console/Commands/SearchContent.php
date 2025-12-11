<?php

namespace App\Console\Commands;

use App\Models\Content;
use Illuminate\Console\Command;

class SearchContent extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'search:content {query : The search query}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Search content using Scout';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $query = $this->argument('query');

        $this->info("Searching for: {$query}");

        $results = Content::search($query)->get();

        if ($results->isEmpty()) {
            $this->warn('No results found.');
            return self::SUCCESS;
        }

        $this->info("Found {$results->count()} result(s):\n");

        foreach ($results as $content) {
            $this->line("ID: {$content->id}");
            $this->line("Title: {$content->title}");
            $this->line("Body: " . substr($content->body, 0, 100) . '...');
            $this->line('---');
        }

        return self::SUCCESS;
    }
}
