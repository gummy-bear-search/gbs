<?php

namespace App\Providers;

use App\Scout\ElasticsearchEngine;
use Illuminate\Support\ServiceProvider;
use Laravel\Scout\EngineManager;

class ScoutServiceProvider extends ServiceProvider
{
    /**
     * Register services.
     */
    public function register(): void
    {
        //
    }

    /**
     * Bootstrap services.
     */
    public function boot(): void
    {
        $this->app->make(EngineManager::class)->extend('elasticsearch', function () {
            return new ElasticsearchEngine();
        });
    }
}
