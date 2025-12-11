<?php

namespace App\Scout;

use Illuminate\Support\Collection;
use Laravel\Scout\Builder;
use Laravel\Scout\Engines\Engine;
use Illuminate\Support\Facades\Http;

class ElasticsearchEngine extends Engine
{
    protected $host;

    public function __construct()
    {
        $this->host = config('scout.elasticsearch.host', env('ELASTICSEARCH_HOST', 'http://localhost:9200'));
    }

    /**
     * Update the given model in the index.
     *
     * @param  \Illuminate\Database\Eloquent\Collection  $models
     * @return void
     */
    public function update($models)
    {
        if ($models->isEmpty()) {
            return;
        }

        $index = $models->first()->searchableAs();
        $operations = [];

        foreach ($models as $model) {
            $operations[] = json_encode([
                'index' => [
                    '_index' => $index,
                    '_id' => $model->getScoutKey(),
                ],
            ]);
            $operations[] = json_encode($model->toSearchableArray());
        }

        $body = implode("\n", $operations) . "\n";

        Http::withBody($body, 'application/x-ndjson')
            ->post("{$this->host}/{$index}/_bulk");
    }

    /**
     * Remove the given model from the index.
     *
     * @param  \Illuminate\Database\Eloquent\Collection  $models
     * @return void
     */
    public function delete($models)
    {
        if ($models->isEmpty()) {
            return;
        }

        $index = $models->first()->searchableAs();
        $operations = [];

        foreach ($models as $model) {
            $operations[] = json_encode([
                'delete' => [
                    '_index' => $index,
                    '_id' => $model->getScoutKey(),
                ],
            ]);
        }

        $body = implode("\n", $operations) . "\n";

        Http::withBody($body, 'application/x-ndjson')
            ->post("{$this->host}/{$index}/_bulk");
    }

    /**
     * Perform the given search on the engine.
     *
     * @param  \Laravel\Scout\Builder  $builder
     * @return mixed
     */
    public function search(Builder $builder)
    {
        return $this->performSearch($builder, [
            'size' => $builder->limit ?? 100,
            'from' => ($builder->page ?? 1) * ($builder->limit ?? 100) - ($builder->limit ?? 100),
        ]);
    }

    /**
     * Perform the given search on the engine.
     *
     * @param  \Laravel\Scout\Builder  $builder
     * @param  int  $perPage
     * @param  int  $page
     * @return mixed
     */
    public function paginate(Builder $builder, $perPage, $page)
    {
        return $this->performSearch($builder, [
            'size' => $perPage,
            'from' => ($page - 1) * $perPage,
        ]);
    }

    /**
     * Perform the given search on the engine.
     *
     * @param  \Laravel\Scout\Builder  $builder
     * @param  array  $options
     * @return array
     */
    protected function performSearch(Builder $builder, array $options = [])
    {
        $index = $builder->model->searchableAs();

        $query = [
            'query' => [
                'match' => [
                    '_all' => $builder->query,
                ],
            ],
        ];

        $response = Http::post("{$this->host}/{$index}/_search", array_merge($query, $options));

        return $response->json() ?? [];
    }

    /**
     * Pluck and return the primary keys of the given results.
     *
     * @param  mixed  $results
     * @return \Illuminate\Support\Collection
     */
    public function mapIds($results)
    {
        return collect($results['hits']['hits'] ?? [])->pluck('_id')->values();
    }

    /**
     * Map the given results to instances of the given model.
     *
     * @param  \Laravel\Scout\Builder  $builder
     * @param  mixed  $results
     * @param  \Illuminate\Database\Eloquent\Model  $model
     * @return \Illuminate\Database\Eloquent\Collection
     */
    public function map(Builder $builder, $results, $model)
    {
        if (empty($results['hits']['hits'])) {
            return $model->newCollection();
        }

        $ids = $this->mapIds($results)->all();

        return $model->getScoutModelsByIds($builder, $ids)
            ->filter(function ($model) use ($ids) {
                return in_array($model->getScoutKey(), $ids);
            })
            ->sortBy(function ($model) use ($ids) {
                return array_search($model->getScoutKey(), $ids);
            })
            ->values();
    }

    /**
     * Get the total count from a raw result returned by the engine.
     *
     * @param  mixed  $results
     * @return int
     */
    public function getTotalCount($results)
    {
        return $results['hits']['total']['value'] ?? 0;
    }

    /**
     * Flush all of the model's records from the engine.
     *
     * @param  \Illuminate\Database\Eloquent\Model  $model
     * @return void
     */
    public function flush($model)
    {
        $index = $model->searchableAs();

        // Delete all documents by searching and deleting each
        // Note: This is a simple implementation. In production, you might want
        // to use scroll API or delete by query when implemented.
        Http::delete("{$this->host}/{$index}");
    }
}
