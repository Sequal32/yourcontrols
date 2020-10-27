import yaml
PREFIX_PREFIX = "YCB_"
SPACING = "    "

yaml_file = open("out/touchscreenkeys.yaml", "w") 
js_file = open("out/TouchScreenKeys.js", "w")

js_file.write("var instrumentButtonMapping = {\n")

count = 0

data = yaml.load(open("touchscreenlist.yaml", "r"), Loader=yaml.Loader)
for entry in data:
    for element in entry["elements"]:
        for prefix in entry["instruments"]:
            yaml_file.write(f"{PREFIX_PREFIX}{prefix}#{element}: {count}\n")
            js_file.write(f"{SPACING}\"{prefix}_{element}\": {count},\n")
            count += 1
            
js_file.write("}")