# Image resizing client: Imagecli

## Description

Imagecli is a basic command-line tool that performs the following two operations:

- Image resize: Resizes one or more images in a source folder to a specified size.

- Image stats: Provides some statistics on the image files present in the src foulder.

## Usage

For printing a help message:

```$> imagecli --help```

For resizing images:

```$> imagecli resize --size small/medium/large --mode all/single --srcfolder <path-to-image-file>``` 

For stats of images: 

```$> imagecli stats --srcfolder <path-to-image-file>```
