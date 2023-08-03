# bk_level

## LEVEL GEOMETRY:

* Open the desired level in Banjo's Backpack
* Close BB
* Grab the file(s) in `<BB>/tmp/`
	* To know which files, look in `<BB>/resources/setups.xml` (`modelAPointer` and `modelBPointer` of the desired level)
* Copy them somewhere else
* Rename them to `<name>.lvl.bin`
* Execute the programme with `<name>.lvl.bin` as the sole argument
* You'll get .obj, .mtl, and .png files

## SETUP FILE:

* Same steps as above but look at the field `pointer`
* Rename the file as `<name>.lvl_setup.bin`
* You'll get `<name>.lvl_setup.yaml`

## REPACK SETUP FILE:

* execute the programme with `<name>.lvl_setup.yaml`
* You'll get `<name>.lvl_setup_repack.bin`
