# fily

fily is a command line tool that combines multiple functions together that can be helpful when dealing with files, especially with a large number of them.

This project is a personal one. There probably are a lot of other programs out there that do similar if not the same things. If you choose to use this understand that I am not making any kind of guarantees about it working or even working correctly. There are definitely bugs in here that I just haven't found yet. I recommend always having logging on and setting the strict logging flag to make sure you can understand what happened if something ever goes wrong.

Now, having said all of that, if you choose to help get this in a state where I'd comfortably be able to remove that notice and need support or have any questions, feel free to make an issue and I'll gladly help you out.

Also, I'm developing this on a Windows machine so this *should* compile on Windows. I can't say the same for Linux since I've never tested it but feel free to try it.

## Building

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Clone the repository
3. In the folder of the repository run `cargo build --release`
4. The executable should appear in `./target/release/fily.exe`

## How to use

`fily` consists of multiple modules that each do different things. You have to specify which one you want to use. Some modules have their own options, you can view the help message for each option with `fily <SUBCOMMAND> --help`.

You can also use `fily -h` to get a list of all modules, global settings and flags. If you set any global settings or flags they have to be __*before*__ the module name. If you put them after the module name they'll be interpreted as options belonging to the module and will be interpreted differently resulting in things you didn't intend.

Currently every module besides `find` expects paths to the files it should work on through stdin. By default it expects them to be separated by a single new line but you can change the separator it uses. The intended usage here is to invoke the `find` module to select the files you want and pipe its output into the module you want to use. For example: `fily find -p "." -i folders | fily duplicates -h`. This command first `find`s every file in the current working directory and every file in every subfolder and then searches it for duplicates.

You can use different ways of getting the paths to the files you want `fily` to work on as long as the paths are either separated by a new line or you specify their separator and they are sent to stdin.

I also recommend that you always enable logging since `fily` will only report errors to stderr if the error causes the whole operation to fail. Otherwise it just logs them. If something didn't quite go like you expected it to or only some files were processed, check the logs.

Also, if you use any operation besides `find` without piping the input it will just wait for input through stdin. You can directly paste what you want in there and then send EOF with Ctrl + D or Ctrl + Z depending on what platform you are on.

## Operations

### find

`find`s files based on criteria you specify. You can add a lot of different criterias that the file has to match. Prints canonicalized paths to the files that match, separated by a new line, to stdout. You can change the separator if you need it.

### rename

`rename`s every file based on a template you provide.

The template is a string that can be passed with `-t`.

Normal text will just be used directly in the name, anything within `{` and `}` will be interpreted as a variable which value can change for every file. The value of the variable will be inserted where the variable was in the template.

You can choose from a couple different variables:

* `filename` The current filename
* `filename_extension` The extension of the filename without the `.`. If there is no extension this will be an empty string
* `filename_base` The base of the filename. If there is no extension this is the same as `filename`
* `filesize` The size of the file in bytes
* `incrementing_number` A number that will increment by one after each file. By default it starts at 0 but you can change the starting point

There are also options for the template. Everything after the first `|` will be interpreted as such. Currently there is only one option:

* `incrementing_number_starts_at` Sets the starting point of `incrementing_number`. The number can be negative. Should be used like this: `{incrementing_number}|incrementing_number_starts_at=42`

### duplicates

Finds exact `duplicates` by checking if a file has the exact same bytes in the same order. This can be very resource expensive. Worst case scenarion is that it has the contents of every file it should check in memory. If you know ahead of time that you don't have enough memory use the `-h` flag. This causes it to hash the contents of the file to a crc32 and only store that. It determines if a file matches by checking if the hashes are equal. This introduces the possibility of a false positive through a hash collision but can reduce the required amount of RAM significantly.

Prints the paths to any duplicate file it found like this `path/to/duplicate1, path/to/duplicate2` to stdout.

### move

`move`s every file to a different folder

### check_image_formats

Checks if the extension of the filename of a picture is the same as its actual format. If it isn't it prints `path/to/file, extension_format, file_format` for every file it found to stdout.

This only works on images. Any other file type may or may not produce an error. For example it'll falsely report `.wav` files as actually being a `.webp` file. You should make sure you only pass paths to images to this.

### similar_images

Finds images that are similar to each other. With this you can find images that have been slightly changed, resulting in `duplicates` not finding it. You can choose which algorithm you want to use and how close they should be to be considered similar.

## Logging

Fily can log everything it does. This is disabled by default but can be set with `-l <log_level>`, `--log_level <log_level>` or with the environment variable `FILY_LOGLEVEL=<log_level>`. There are 6 different levels. They are, in order of increasing importance:

* `off`
* `trace`
* `debug`
* `info`
* `warn`
* `error`

Setting it to one level automatically includes all levels that have a higher importance. For example, setting the log level to `info` will also log any events that have the levels `warn` or `error`. `trace` will give you the most information but isn't needed for most people. If you just want to know which actions the program is taking set it to `info`.

You always have to set the loglevel before specifying the operation. For example:

 `fily -l trace find -p "a/path" -d 2 | fily -l info -s rename -t "{incrementing_number}|incrementing_number_starts_at=5"`

Having it at any position behind the operation name will not result in what you want.

There is also the strict logging flag. You can enable it with `-s`, `--strict_logging` or by creating an environment variable called `FILY_STRICT_LOGGING`. The value of the environment variable doesn't matter. Setting this flag doesn't allow fily to run without logging. If the setup fails for some reason the program will just stop before doing anything. This does not stop the program if you set logging to `off`. This only makes sure that the setup does not fail.

## Dealing with UTF-8 in PowerShell on Windows

Turns out that PowerShell on Windows isn't set to UTF-8 by default to support legacy console applications. This turns into a problem if any path contains non-ASCII characters because it'll just change those characters to a literal `?`, resulting in broken paths.

If you want to change the encoding to UTF-8 paste this into PowerShell:

`$OutputEncoding = [console]::InputEncoding = [console]::OutputEncoding = New-Object System.Text.UTF8Encoding`

This will only change the encoding for this session, if you want to know how to change it permanently or just want to read more about this look at these SO questions.

[https://stackoverflow.com/questions/49476326/displaying-unicode-in-powershell](https://stackoverflow.com/questions/49476326/displaying-unicode-in-powershell)

[https://stackoverflow.com/questions/57131654/using-utf-8-encoding-chcp-65001-in-command-prompt-windows-powershell-window/57134096](https://stackoverflow.com/questions/57131654/using-utf-8-encoding-chcp-65001-in-command-prompt-windows-powershell-window/57134096)