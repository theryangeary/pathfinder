#!/bin/bash

echo "=== Word Game Integration Test ==="
echo

# Test backend API endpoints
echo "1. Testing daily game endpoint..."
response=$(curl -s http://localhost:3001/api/daily-game)
if echo "$response" | grep -q '"id"'; then
    echo "✓ Daily game API working"
    game_id=$(echo "$response" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    echo "  Game ID: ${game_id:0:8}..."
else
    echo "✗ Daily game API failed"
    echo "Response: $response"
fi

echo

echo "2. Testing user creation endpoint..."
user_response=$(curl -s -X POST http://localhost:3001/api/user)
if echo "$user_response" | grep -q '"user_id"'; then
    echo "✓ User creation API working"
    user_id=$(echo "$user_response" | grep -o '"user_id":"[^"]*"' | cut -d'"' -f4)
    echo "  User ID: ${user_id:0:8}..."
else
    echo "✗ User creation API failed"
    echo "Response: $user_response"
fi

echo

echo "3. Testing word validation endpoint..."
validation_payload='{"word":"test","previous_answers":[]}'
validation_response=$(curl -s -X POST -H "Content-Type: application/json" -d "$validation_payload" http://localhost:3001/api/validate)
if echo "$validation_response" | grep -q '"is_valid"'; then
    echo "✓ Word validation API working"
    is_valid=$(echo "$validation_response" | grep -o '"is_valid":[^,]*' | cut -d':' -f2)
    echo "  'test' is valid: $is_valid"
else
    echo "✗ Word validation API failed"
    echo "Response: $validation_response"
fi

echo

echo "4. Checking frontend accessibility..."
if curl -s http://localhost:5174/ > /dev/null; then
    echo "✓ Frontend is accessible on http://localhost:5174"
else
    echo "✗ Frontend not accessible"
fi

echo

echo "5. Checking backend logs for errors..."
if [ -f src/api/backend.log ]; then
    error_count=$(grep -i error src/api/backend.log | wc -l)
    if [ "$error_count" -eq 0 ]; then
        echo "✓ No errors in backend logs"
    else
        echo "⚠ Found $error_count errors in backend logs:"
        grep -i error src/api/backend.log | tail -3
    fi
else
    echo "⚠ Backend log file not found"
fi

echo
echo "=== Integration Test Complete ==="
echo
echo "Next steps:"
echo "1. Open http://localhost:5174 in your browser"
echo "2. Verify the game loads and shows a daily puzzle"
echo "3. Try entering words and check validation"
echo "4. Complete a full game and submit answers"