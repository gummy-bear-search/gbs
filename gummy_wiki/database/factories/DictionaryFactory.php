<?php

namespace Database\Factories;

use App\Models\Dictionary;
use Illuminate\Database\Eloquent\Factories\Factory;

class DictionaryFactory extends Factory
{
    protected $model = Dictionary::class;

    public function definition(): array
    {
        return [
            'term' => $this->faker->word(),
            'definition' => $this->faker->sentence(),
            'language' => $this->faker->randomElement(['en', 'ru', 'es', 'fr']),
        ];
    }
}
