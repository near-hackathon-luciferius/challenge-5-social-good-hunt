import React from 'react';
import { Button } from 'react-materialize';

export default function Register({onRegister}) {
  return (
    <>
      <header>
                   <h1>Social Bounty Hunt Homepage</h1>
      </header>
      <Button small onClick={onRegister}>Register</Button>
      <p>
          To use the DEED token you must be registered to the smart contract.
          Without that you cannot use it. There is a small fee to pay for the 
          registration in order to pay for the used storage.
      </p>
      <p>
          Go ahead and register to finally try the app!
      </p>
    </>
  );
}
