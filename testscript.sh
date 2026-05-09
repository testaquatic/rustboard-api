#!/bin/bash
# test_auth.sh — rustboard 인증 통합 테스트
set -euo pipefail
BASE_URL="http://localhost:3000"
echo "=== 1. 회원가입 ==="
curl -s -X POST "$BASE_URL/signup" \
  -H 'Content-Type: application/json' \
  -d '{"email":"alice@test.com","password":"pass1234","display_name":"Alice"}'
echo ""
curl -s -X POST "$BASE_URL/signup" \
  -H 'Content-Type: application/json' \
  -d '{"email":"bob@test.com","password":"pass5678","display_name":"Bob"}'
echo ""
echo "=== 2. 로그인 ==="
ALICE_TOKEN=$(curl -s -X POST "$BASE_URL/login" \
  -H 'Content-Type: application/json' \
  -d '{"email":"alice@test.com","password":"pass1234"}' \
  | jq -r '.token')
echo "Alice token: ${ALICE_TOKEN:0:20}..."
BOB_TOKEN=$(curl -s -X POST "$BASE_URL/login" \
  -H 'Content-Type: application/json' \
  -d '{"email":"bob@test.com","password":"pass5678"}' \
  | jq -r '.token')
echo "Bob token: ${BOB_TOKEN:0:20}..."
echo ""
echo "=== 3. 비인증 글 작성 시도 → 401 ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  -X POST "$BASE_URL/posts" \
  -H 'Content-Type: application/json' \
  -d '{"title":"No Token","content":"이건 실패해야 함"}')
echo "Expected 401, Got: $HTTP_CODE"
echo ""
echo "=== 4. Alice가 글 작성 → 201 ==="
ALICE_POST=$(curl -s -X POST "$BASE_URL/posts" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Alice의 글","content":"안녕하세요"}')
echo "$ALICE_POST" | jq .
POST_ID=$(echo "$ALICE_POST" | jq -r '.id')
echo ""
echo "=== 5. 비인증 글 조회 → 200 (공개) ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/posts/$POST_ID")
echo "Expected 200, Got: $HTTP_CODE"
echo ""
echo "=== 6. Bob이 Alice의 글 수정 시도 → 403 ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  -X PATCH "$BASE_URL/posts/$POST_ID" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Bob이 수정","content":"이건 실패해야 함"}')
echo "Expected 403, Got: $HTTP_CODE"
echo ""
echo "=== 7. Alice가 자기 글 수정 → 200 ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  -X PATCH "$BASE_URL/posts/$POST_ID" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Alice가 수정","content":"수정된 내용"}')
echo "Expected 200, Got: $HTTP_CODE"
echo ""
echo "=== 8. Bob이 Alice의 글 삭제 시도 → 403 ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  -X DELETE "$BASE_URL/posts/$POST_ID" \
  -H "Authorization: Bearer $BOB_TOKEN")
echo "Expected 403, Got: $HTTP_CODE"
echo ""
echo "=== 9. Alice가 자기 글 삭제 → 204 ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  -X DELETE "$BASE_URL/posts/$POST_ID" \
  -H "Authorization: Bearer $ALICE_TOKEN")
echo "Expected 204, Got: $HTTP_CODE"
echo ""
echo "=== 10. 틀린 비밀번호 로그인 → 422 ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  -X POST "$BASE_URL/login" \
  -H 'Content-Type: application/json' \
  -d '{"email":"alice@test.com","password":"wrong"}')
echo "Expected 422, Got: $HTTP_CODE"
echo ""
echo "=== 완료 ==="