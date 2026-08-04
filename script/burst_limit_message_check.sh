for i in $(seq 1 40); do
    curl -s http://localhost:3000/posts > /dev/null
done

curl -i http://localhost:3000/posts