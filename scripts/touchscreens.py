import os
import yaml
PREFIX_PREFIX = "YCB_"
SPACING = "    "

yaml_file = open("out/touchscreenkeys.yaml", "w") 
js_file = open("out/TouchScreenKeys.js", "w")

js_file.write("var instrumentButtonMapping = {\n")

count = 0

for filename in os.listdir("touchscreens/"):
    data = yaml.load(open("touchscreens/" + filename, "r"), Loader=yaml.Loader)
    for entry in data:
        for element in entry["elements"]:
            for prefix in entry["instruments"]:
                yaml_file.write(f"{PREFIX_PREFIX}{prefix}#{element}: {count}\n")
                js_file.write(f"{SPACING}\"{prefix}_{element}\": {count},\n")
                count += 1
            
js_file.write("}")