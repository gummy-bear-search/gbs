<?php

namespace App\Console\Commands;

use Database\Seeders\WikipediaSeeder;
use Illuminate\Console\Command;

class SeedWikipedia extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'seed:wikipedia
                            {--count=50 : Number of articles to fetch}
                            {--content-only : Only seed Content model}
                            {--entities-only : Only seed Entity model}
                            {--dictionary-only : Only seed Dictionary model}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Seed database with real data from Wikipedia';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $this->info('Starting Wikipedia seeding...');
        $this->newLine();

        $seeder = new WikipediaSeeder();

        // Override article count if specified
        if ($this->option('count') != 50) {
            // Note: This would require modifying the seeder to accept count
            $this->warn('Custom count not yet implemented. Using default article list.');
        }

        // Run seeder with command context
        $seeder->command = $this;
        $seeder->run();

        $this->newLine();
        $this->info('Wikipedia seeding completed successfully!');
        $this->info('Next steps:');
        $this->line('  1. Run: php artisan elastic:create-index');
        $this->line('  2. Run: php artisan scout:import');

        return Command::SUCCESS;
    }
}
