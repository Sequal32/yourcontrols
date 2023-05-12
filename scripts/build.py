from zipfile import ZipFile, ZIP_DEFLATED
from os.path import basename
import os


def insert_all_into_zip(zip_obj: ZipFile, directory: str, relative_dir: str = ""):
    # Iterate over all the files in directory
    for folder_name, subfolders, filenames in os.walk(directory):
        for filename in filenames:
            # create complete filepath of file in directory
            file_path = os.path.join(folder_name, filename)
            # Add file to zip
            zip_obj.write(
                file_path,
                os.path.join(
                    relative_dir, os.path.relpath(folder_name, directory), filename
                ),
            )

if not os.path.exists("scripts/out"):
    os.mkdir("scripts/out")

zipObj = ZipFile("scripts/out/YourControls.zip", "w", ZIP_DEFLATED)

insert_all_into_zip(zipObj, "definitions/aircraft", "definitions/aircraft")
insert_all_into_zip(zipObj, "definitions/modules", "definitions/modules")
insert_all_into_zip(zipObj, "scripts/build-include", "")
zipObj.write("assets/logo.png", "assets/logo.png")
zipObj.write("assets/disconnected.mp3", "assets/disconnected.mp3")
zipObj.write("target/release/YourControls.exe", "YourControls.exe")
zipObj.write("SimConnect.dll", "SimConnect.dll")

zipObj.close()
