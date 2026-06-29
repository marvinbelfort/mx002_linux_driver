all:
	cargo build
run:
	echo "Driver[mx022] requires sudo"
	sudo ./target/debug/mx002

config_py:

# 2. Crie o ambiente virtual (uma mini-instalação isolada do Python)
	python3 -m venv venv

# 3. Ative o ambiente (o terminal vai ganhar um prefixo '(venv)')
	source venv/bin/activate

# 4. Agora o pip tradicional vai funcionar perfeitamente!
	pip install pillow pyautogui