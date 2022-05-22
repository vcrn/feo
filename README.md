# FeO

FeO (feo) is a very simple system monitoring CLI tool for Linux systems written by <a href="https://github.com/vcrn">@vcrn</a>. 

FeO monitors the CPU temperature, CPU load, RAM and swap usage, and uptime. GPU temperature monitoring available for Raspberry Pi. FeO is written in Rust and inspired by the process monitoring tool htop. Named after a part of the chemical formulas for rust, with the bonus of 'feo' meaning ugly in Spanish.

<pre>
USAGE:
    feo [OPTIONS]
    
OPTIONS:  
    -c, --color &lt;COLOR&gt;    Select color-scheme for monitor: 'w' for white, 'b' for black, 's' for standard [default: s]
    -d, --delay &lt;DELAY&gt;    Set the delay between updates, in whole seconds [default: 2]  
    -g, --gpu              Monitor the GPU temperature  
    -h, --help             Print help information  
    -V, --version          Print version information  
</pre>
