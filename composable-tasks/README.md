# Composable Tasks SC

## Overview

This smart contract enables users to compose multiple actions while interacting with various Smart Contracts from Dharitri ecosystem, including xExchange.
It streamlines the process of interacting with WrapMOAX and xExchange and provides a convenient way to perform multiple actions in a single transaction on the blockchain.

Complex actions are formed of multiple tasks. The tasks are performed synchronously, one after the other.
Example of tasks:
- wrapMOAX
- unwrapMOAX
- Swap
- Send MOAX/DCT to third party


Example of actions:
- Wrap MOAX & send to third party
- Swap DCT to wMOAX & unwrap to MOAX
- Wrap MOAX & swap to DCT & send to third party

> **_Note:_** If the last task is **not** `Send tokens`, the resulted payment will be returned to the caller. Otherwise, the payment goes to the destination. 

## Task Structure

A task receives an `MoaxOrDctPayment` and outputs one as well.
The resulted `MoaxOrDctPayment` is forwarded to the next task.
If one task fails, the whole process will fail.

The `composeTasks` endpoint:
```
    #[payable("*")]
    #[endpoint(composeTasks)]
    fn compose_tasks(
        &self,
        opt_dest_addr: OptionalValue<ManagedAddress>,
        tasks: MultiValueEncoded<MultiValue2<TaskType, ManagedVec<ManagedBuffer>>>,
    )
```

where `TaskType`:

```
pub enum TaskType {
    WrapMOAX,
    UnwrapMOAX,
    Swap,
    SendDct,
}
```


> **_WARNING:_**  If you provide a wrong destination address, the payment will be sent there.

Most of the tasks don't require arguments, but some do (like `Swap`). An example of calling `Swap` task:

```
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(TOKEN_ID));
                swap_args.push(managed_buffer!(b"1"));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::Swap, swap_args).into());
```