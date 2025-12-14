<?php

namespace Tests\Feature;

use App\Models\Content;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class SearchTest extends TestCase
{
    use RefreshDatabase;

    public function test_can_search_content(): void
    {
        // Create test content
        $content = Content::create([
            'title' => 'Test Article',
            'body' => 'This is a test article about Laravel and Scout.',
        ]);

        // Index the content
        $content->searchable();

        // Wait a bit for indexing (in real scenario, you'd use refresh)
        sleep(1);

        // Search for content
        $results = Content::search('Laravel')->get();

        $this->assertGreaterThan(0, $results->count());
    }

    public function test_can_create_and_index_content(): void
    {
        $content = Content::create([
            'title' => 'New Article',
            'body' => 'Content body here',
        ]);

        // This should automatically index when using Scout events
        $content->searchable();

        $this->assertDatabaseHas('contents', [
            'id' => $content->id,
            'title' => 'New Article',
        ]);
    }
}
