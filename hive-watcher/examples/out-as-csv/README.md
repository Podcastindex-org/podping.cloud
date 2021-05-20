# This example
The intent here is to create a data.csv plain text file for quick data exploration of the podping data being reported from the hive blockchain.

# Dependancies
- Linux (install depenencies examples here are for apt (debian based distros)
- Ensure python3 and pip are installed (check with `pip --version`)
- Install beem wit pip
```
pip3 install beem
```
- Install R
```
sudo apt install r-base r-base-core r-recommended
```
# Running the scripts
## Everything is a script
Some automation - make these scripts executable
- navigate to this example's folder and run:
```
sudo chmod +x *.py *.R
```
## Collect data
Then to begin or resume collecting data run:
```
./hive-watcher-simple.py
```
## Visualize it
After running for awhile run the data visualization script with:
```
./visualize-data.R 
```

## Results
What's the delay between the watcher and the posted time stamp on the hive blockchain?
Here are some descriptive visualizations.
![Example Histogram](stats/image-timestamp_delay_hist.png)
![Example Scatterplot](stats/image-timestamp_delay.png)