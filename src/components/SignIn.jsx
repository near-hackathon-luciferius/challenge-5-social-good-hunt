import React from 'react';
import { Button } from 'react-materialize';

export default function SignIn({signIn}) {
  return (
    <>
      <header>
                   <h1>Social Bounty Hunt Homepage</h1>
      </header>
      <Button small onClick={signIn}>Log in</Button>
      <p>
          This app demonstrates how to mint nfts in with the NEAR blockchain. While minting
          the app will ask you to deposite 0.1 NEAR but it acutally only uses roughly 0.01 NEAR.
          The remaining NEAR that is not used gets refunded on completing the smart contrat call.
      </p>
      <p>
          Go ahead and sign in to try it out!
      </p>
    </>
  );
}
