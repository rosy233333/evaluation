import matplotlib.pyplot as plt
import numpy as np
import seaborn as sns

def get_data(filename):
    with open(filename, 'r') as file:
        str = file.read().split(', ')
        data = [int(i) for i in str]
        data = [x for x in data if x < 160000 and x > 40000] # for graph
        return data

if __name__ == "__main__":
    async_trait_read_time = get_data("async-std-result/async_trait_read_out.txt")
    async_trait_read_std = np.std(async_trait_read_time)
    async_trait_read_avg = np.mean(async_trait_read_time)
    print("async_trait_read_avg: ", async_trait_read_avg, " async_trait_read_std: ", async_trait_read_std)

    stack_future_read_time = get_data("async-std-result/stack_future_read_out.txt")
    stack_future_read_std = np.std(stack_future_read_time)
    stack_future_read_avg = np.mean(stack_future_read_time)
    print("stack_future_read_avg: ", stack_future_read_avg, " stack_future_read_std: ", stack_future_read_std)

    static_dispatch_read_time = get_data("async-std-result/static_dispatch_read_out.txt")
    static_dispatch_read_std = np.std(static_dispatch_read_time)
    static_dispatch_read_avg = np.mean(static_dispatch_read_time)
    print("static_dispatch_read_avg: ", static_dispatch_read_avg, " static_dispatch_read_std: ", static_dispatch_read_std)

    afit_static_dispatch_read_time = get_data("async-std-result/afit_static_dispatch_read_out.txt")
    afit_static_dispatch_read_std = np.std(afit_static_dispatch_read_time)
    afit_static_dispatch_read_avg = np.mean(afit_static_dispatch_read_time)
    print("afit_static_dispatch_read_avg: ", afit_static_dispatch_read_avg, " afit_static_dispatch_read_std: ", afit_static_dispatch_read_std)

    dynosaur_read_time = get_data("async-std-result/dynosaur_read_out.txt")
    dynosaur_read_std = np.std(dynosaur_read_time)
    dynosaur_read_avg = np.mean(dynosaur_read_time)
    print("dynosaur_read_avg: ", dynosaur_read_avg, " dynosaur_read_std: ", dynosaur_read_std)

    sns.kdeplot(async_trait_read_time, color="purple", label="async_trait", fill=True, bw_adjust=5, multiple = "layer")
    sns.kdeplot(stack_future_read_time, color="green", label="stack_future", fill=True, bw_adjust=5, multiple = "layer")
    sns.kdeplot(static_dispatch_read_time, color="red", label="static_dispatch", fill=True, bw_adjust=5, multiple = "layer")
    sns.kdeplot(afit_static_dispatch_read_time, color="yellow", label="afit_static_dispatch", fill=True, bw_adjust=5, multiple = "layer")
    sns.kdeplot(dynosaur_read_time, color="blue", label="dynosaur", fill=True, bw_adjust=5, multiple = "layer")
    plt.title('Probability Density Function (PDF)')
    plt.xlabel('Value')
    plt.autoscale(enable=True, axis='x', tight=None)
    plt.ylabel('Density')
    plt.xlim(40000, 160000)
    plt.savefig('test.png')
    plt.show()


