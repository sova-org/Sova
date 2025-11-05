# Troubleshoot

There are some common issues that you might encounter while using the software. Here are some solutions to help you resolve them:

## Installation Issues

### Binary not found +

If you get an error that the `sova` binary is not found, make sure you have added the installation directory to your PATH.

### Permission denied +

On macOS/Linux, you may need to make the binary executable:
```bash
chmod +x /path/to/sova
```

## Connection Issues

### OSC Port Already in Use +

If you see "Address already in use" errors, this typically means you already have a running Sova/Sardine session. Check for existing processes before starting a new one.

### MIDI Device Not Found +

Ensure your MIDI devices are properly connected and recognized by your operating system before launching Sova.

## Runtime Issues

### Code Not Executing +

- Verify your syntax is correct
- Check the console for error messages
- Ensure you're in the correct editing mode

### Performance Issues +

- Try reducing the number of active patterns
- Check CPU usage in your system monitor
- Consider increasing buffer sizes in your audio settings

## Audio Issues

### No Sound Output +

- Verify your audio interface is selected correctly
- Check volume levels in your system mixer
- Ensure SuperCollider/audio engine is running

### Audio Glitches or Dropouts +

- Increase buffer size in audio settings
- Close unnecessary applications
- Check for system resource constraints

## Getting Help

### Still having issues? +

If you're still experiencing issues:

- Check the [GitHub issues](https://github.com/Bubobubobubobubo/sova/issues)
- Join the community discussion
- Provide error messages and system information when reporting bugs
