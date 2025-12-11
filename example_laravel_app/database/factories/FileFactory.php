<?php

namespace Database\Factories;

use App\Models\File;
use Illuminate\Database\Eloquent\Factories\Factory;

class FileFactory extends Factory
{
    protected $model = File::class;

    public function definition(): array
    {
        return [
            'filename' => $this->faker->word() . '.' . $this->faker->fileExtension(),
            'path' => '/files/' . $this->faker->uuid() . '/' . $this->faker->word() . '.' . $this->faker->fileExtension(),
            'mime_type' => $this->faker->mimeType(),
            'size' => $this->faker->numberBetween(1024, 10485760), // 1KB to 10MB
            'description' => $this->faker->optional()->sentence(),
        ];
    }
}
