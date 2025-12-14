<?php

namespace Database\Seeders;

use App\Models\Content;
use App\Models\Dictionary;
use App\Models\Entity;
use App\Models\File;
use Illuminate\Database\Seeder;

class DatabaseSeeder extends Seeder
{
    /**
     * Seed the application's database.
     */
    public function run(): void
    {
        // Check if we should use Wikipedia data
        $useWikipedia = env('USE_WIKIPEDIA_SEEDER', false);

        if ($useWikipedia) {
            $this->call(WikipediaSeeder::class);

            // Still create some files with factory
            File::factory(10)->create();
        } else {
            // Use factory-generated fake data
            Content::factory(20)->create();
            Entity::factory(15)->create();
            File::factory(10)->create();
            Dictionary::factory(25)->create();
        }
    }
}
