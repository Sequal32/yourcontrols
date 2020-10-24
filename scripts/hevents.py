# Compiles .yaml and .js from an array of .txt
SPACING = "    "

yaml_file = open("hevents.yaml", "w") 
js_file = open("keys.js", "w")

js_file.write("var hEventKeyMapping = {\n")

with open("hevents.txt", "r") as f:
    for i, line in enumerate(f):
        line = line.strip().upper()
        # Skip comments
        if line.find("//") != -1: continue
        
        yaml_file.write(f"{line}: {i}\n")
        js_file.write(f"{SPACING}\"{line}\": {i},\n")
        
js_file.write("}")