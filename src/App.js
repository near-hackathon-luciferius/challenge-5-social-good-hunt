import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';
import Big from 'big.js';
import SignIn from './components/SignIn';
import Layout from './layout';
import NotFound from './components/404.jsx';
import Dashboard from './components/Dashboard.jsx';
import Publish from './components/Publish.jsx';
import Overview from './components/Overview.jsx';
import Donate from './components/Donate.jsx';
import Register from './components/Register.jsx';
import 'materialize-css/dist/css/materialize.css'
import './App.css';
import { Route, Routes } from 'react-router-dom'
var version = require('../package.json').version;
require('materialize-css');

const BOATLOAD_OF_GAS = Big(3).times(10 ** 13).toFixed();

const App = ({ contract, currentUser, nearConfig, wallet, provider, lastTransaction, error }) => {
  const [message, setMessage] = useState('');
  const [registered, setRegistered] = useState(false);

  const onPublishDeed = async (e) => {
    e.preventDefault();

    const { fieldset, proof_prompt, title_prompt, description_prompt } = e.target.elements;
    
    fieldset.disabled = true;

    contract.add_deed(
      {
        author: currentUser.accountId,
        title: title_prompt.value,
        description: description_prompt.value,
        proof: proof_prompt.value
      },
      BOATLOAD_OF_GAS,
      Big('0.01').times(10 ** 24).toFixed()
    ).then((_) => {
      console.log("Successfully published.");
    })
  };

  const onCredit = async (deed) => {
    if(deed.is_creditor){
      setMessage(`You already credited deed #${deed.id}`);
      return;
    }

    contract.credit(
      {
        id: deed.id
      },
      BOATLOAD_OF_GAS,
      Big('0.002').times(10 ** 24).toFixed()
    ).then((_) => {
      console.log("Successfully credited.");
    })
  };

  const onRegister = async (_) => {
    const balance_bounds = await contract.storage_balance_bounds();
    contract.storage_deposit(
      {
        account_id: currentUser.accountId,
        registration_only: true
      },
      BOATLOAD_OF_GAS,
      Big(balance_bounds.min).toFixed()
    ).then((_) => {
      console.log("Successfully registered.");
    })
  };

  const onDonate = async (e) => {
    e.preventDefault();

    const { fieldset, donation } = e.target.elements;
    
    fieldset.disabled = true;

    contract.donate(
      { },
      BOATLOAD_OF_GAS,
      Big(donation.value).times(10 ** 24).toFixed()
    ).then((_) => {
      console.log("Successfully donated.");
    })
  };
  
  useEffect(() => {      
    const fetchRegistered = async () => {
      const isRegistered = await contract.is_registered({account_id: currentUser.accountId});
      setRegistered(isRegistered);
    }

    fetchRegistered();
  }, [contract, currentUser]);
  
  useEffect(() => {
      if (error){
        setMessage(decodeURI(error));
        window.history.pushState({}, "", window.location.origin + window.location.pathname);
      }
      else if (lastTransaction) {          
          setMessage(`Successfully executed transaction ${lastTransaction}`);
          window.history.pushState({}, "", window.location.origin + window.location.pathname);
      }
  }, [lastTransaction, error]);
  
  const signIn = () => {
    wallet.requestSignIn(
      {contractId: nearConfig.contractName, methodNames: [contract.add_deed.name, contract.credit.name, contract.donate.name]}, //contract requesting access
      'NEAR Challenge #5 - Social Bounty Hunt', //optional name
      null, //optional URL to redirect to if the sign in was successful
      null //optional URL to redirect to if the sign in was NOT successful
    );
  };

  const signOut = () => {
    wallet.signOut();
    window.location.replace(window.location.origin + window.location.pathname);
  };

  const clearMessage = () => {
    setMessage('');
  };

  return (
    <Routes>
      <Route path="/" element={<Layout currentUser={currentUser} signIn={signIn} signOut={signOut} clearMessage={clearMessage} message={message}/>}>
        <Route index element={
          currentUser
            ? registered
                ? <Dashboard version={version} currentUser={currentUser}/>
                : <Register onRegister={onRegister} />
            : <SignIn signIn={signIn}/>
        }/>
        <Route path="publish" element={
          currentUser
            ? registered
                ? <Publish onPublishDeed={onPublishDeed}/>
                : <Register onRegister={onRegister} />
            : <SignIn signIn={signIn}/>
        }/>
        <Route path="overview" element={
          currentUser
            ? registered
                ? <Overview currentUser={currentUser} contract={contract} onCredit={onCredit}/>
                : <Register onRegister={onRegister} />
            : <SignIn signIn={signIn}/>
        }/>
        <Route path="donate" element={
          currentUser
            ? registered
                ? <Donate onDonate={onDonate}/>
                : <Register onRegister={onRegister} />
            : <SignIn signIn={signIn}/>
        }/>
        <Route path="*" element={<NotFound/>}/>
      </Route>
    </Routes>
  );
}

App.propTypes = {
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  }),
  nearConfig: PropTypes.shape({
    contractName: PropTypes.string.isRequired
  }).isRequired,
  wallet: PropTypes.shape({
    requestSignIn: PropTypes.func.isRequired,
    signOut: PropTypes.func.isRequired
  }).isRequired
};

export default App;
