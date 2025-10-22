mkdir -p output
cd output


echo "Running compiler"
cargo run --manifest-path ../../../../main/Cargo.toml --bin ants

echo "Generating images"
for f in *.dot; do dot -Tpng "$f" -o "${f%.dot}.png"; done
echo "Done"

rm *.dot
