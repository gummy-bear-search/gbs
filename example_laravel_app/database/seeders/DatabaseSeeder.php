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
        // Create sample content
        Content::factory(20)->create();

        // Create sample entities
        Entity::factory(15)->create();

        // Create sample files
        File::factory(10)->create();

        // Create sample dictionary entries
        Dictionary::factory(25)->create();
    }
}
