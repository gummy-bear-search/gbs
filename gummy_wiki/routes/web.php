<?php

use Illuminate\Support\Facades\Route;

Route::get('/', function () {
    return response()->json([
        'message' => 'Gummy Search Laravel Example API',
        'version' => '1.0.0',
    ]);
});
