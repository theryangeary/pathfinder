#!/bin/bash

echo "=== Word Game Integration Test ==="
echo

# Track test failures
FAILED_TESTS=0

# Test backend health endpoint
echo "1. Testing health endpoint..."
health_response=$(curl -s -H "Referer: http://localhost:5173" http://localhost:3001/health)
if echo "$health_response" | grep -q '"status":"healthy"'; then
    echo "✓ Health endpoint working"
else
    echo "✗ Health endpoint failed"
    echo "Response: $health_response"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

echo

# Test user creation endpoint
echo "2. Testing user creation endpoint..."
user_response=$(curl -s -X POST -H "Referer: http://localhost:5173" http://localhost:3001/api/user)
if echo "$user_response" | grep -q '"user_id"' && echo "$user_response" | grep -q '"cookie_token"'; then
    echo "✓ User creation API working"
    user_id=$(echo "$user_response" | grep -o '"user_id":"[^"]*"' | cut -d'"' -f4)
    cookie_token=$(echo "$user_response" | grep -o '"cookie_token":"[^"]*"' | cut -d'"' -f4)
    echo "  User ID: ${user_id:0:8}..."
    echo "  Cookie Token: ${cookie_token:0:8}..."
else
    echo "✗ User creation API failed"
    echo "Response: $user_response"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

echo

# Test game by date endpoint (today's date)
echo "3. Testing daily game endpoint..."
today=$(date +%Y-%m-%d)
game_response=$(curl -s -H "Referer: http://localhost:5173" http://localhost:3001/api/game/date/$today)
if echo "$game_response" | grep -q '"id"' && echo "$game_response" | grep -q '"board"'; then
    echo "✓ Daily game API working"
    game_id=$(echo "$game_response" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    echo "  Game ID: ${game_id:0:8}..."
    echo "  Date: $today"
else
    echo "✗ Daily game API failed"
    echo "Response: $game_response"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

echo

# Test word validation endpoint
echo "4. Testing word validation endpoint..."
validation_payload='{"word":"test","previous_answers":[]}'
validation_response=$(curl -s -X POST -H "Content-Type: application/json" -H "Referer: http://localhost:5173" -d "$validation_payload" http://localhost:3001/api/validate)
if echo "$validation_response" | grep -q '"is_valid"'; then
    echo "✓ Word validation API working"
    is_valid=$(echo "$validation_response" | grep -o '"is_valid":[^,}]*' | cut -d':' -f2)
    echo "  'test' is valid: $is_valid"
else
    echo "✗ Word validation API failed"
    echo "Response: $validation_response"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

echo

# Test game words endpoint (if we have a game ID)
if [ -n "$game_id" ]; then
    echo "5. Testing game words endpoint..."
    words_response=$(curl -s -H "Referer: http://localhost:5173" http://localhost:3001/api/game/$game_id/words)
    if echo "$words_response" | grep -q '\['; then
        echo "✓ Game words API working"
        word_count=$(echo "$words_response" | grep -o '"[^"]*"' | wc -l)
        echo "  Found $word_count words"
    else
        echo "✗ Game words API failed"
        echo "Response: $words_response"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    echo
    
    echo "6. Testing game paths endpoint..."
    paths_response=$(curl -s -H "Referer: http://localhost:5173" http://localhost:3001/api/game/$game_id/paths)
    if echo "$paths_response" | grep -q '"words"'; then
        echo "✓ Game paths API working"
        echo "  Paths response structure valid"
    else
        echo "✗ Game paths API failed"
        echo "Response: $paths_response"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    echo
    
    echo "7. Testing game entry endpoint..."
    if [ -n "$user_id" ] && [ -n "$cookie_token" ]; then
        entry_response=$(curl -s -H "Referer: http://localhost:5173" "http://localhost:3001/api/game-entry/$game_id?user_id=$user_id&cookie_token=$cookie_token")
        if echo "$entry_response" | grep -q 'null' || echo "$entry_response" | grep -q '"answers"'; then
            echo "✓ Game entry API working"
            echo "  Entry response valid (no existing entry or valid entry)"
        else
            echo "✗ Game entry API failed"
            echo "Response: $entry_response"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        echo "⚠ Skipping game entry test (no user credentials)"
    fi
else
    echo "⚠ Skipping game-specific tests (no game ID)"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

echo

echo "8. Checking backend logs for errors..."
if [ -f src/api/backend.log ]; then
    error_count=$(grep -i error src/api/backend.log | wc -l)
    if [ "$error_count" -eq 0 ]; then
        echo "✓ No errors in backend logs"
    else
        echo "⚠ Found $error_count errors in backend logs:"
        grep -i error src/api/backend.log | tail -3
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
else
    echo "⚠ Backend log file not found"
fi

echo
echo "=== Integration Test Complete ==="
echo

if [ $FAILED_TESTS -eq 0 ]; then
    echo "✅ All integration tests passed!"
    echo
    echo "API endpoints tested:"
    echo "  - GET /health"
    echo "  - POST /api/user"
    echo "  - GET /api/game/date/:date"
    echo "  - POST /api/validate"
    echo "  - GET /api/game/:game_id/words"
    echo "  - GET /api/game/:game_id/paths"
    echo "  - GET /api/game-entry/:game_id"
    exit 0
else
    echo "❌ $FAILED_TESTS integration test(s) failed!"
    echo "Please fix the failing tests before proceeding."
    exit 1
fi
