for i in $(seq 1 40); do
  echo -n "Request $i: "
  curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/posts
  echo
done