import yaml
# Compiles .yaml and .js from an YAML array
SPACING = "    "

yaml_file = open("out/hevents.yaml", "w") 
js_file = open("out/Keys.js", "w")

js_file.write("var hEventKeyMapping = {\n")

data = yaml.load(open("heventlist.yaml", "r"), Loader=yaml.Loader)
for i, line in enumerate(data):
    line = line.strip().upper()
    # Skip comments
    if line.find("//") != -1 or line.strip() == "": continue
    
    yaml_file.write(f"{line}: {i}\n")
    js_file.write(f"{SPACING}\"{line}\": {i},\n")
        
js_file.write("}")